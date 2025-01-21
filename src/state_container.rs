use std::mem::transmute;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::Instant;
use std::{future::poll_fn, sync::Mutex};

use crate::time::{SpawnAtTask, spawn_at};
use crate::utils::Action;
use ::futures::{Stream, stream};
use derive_ex::Ex;

use crate::utils::bipartite_graph::*;
use crate::utils::inf_vec::*;
use crate::utils::usize_set::USizeSet;

#[derive(Debug)]
struct StateGraph {
    g: BipartiteGraph,
    wakers: InfVec<Option<Action>>,
    wake_at: Option<Instant>,
    source_set: USizeSet,
    source_remove: Arc<Mutex<Vec<XKey>>>,
}
impl StateGraph {
    pub fn new() -> Self {
        Self {
            g: BipartiteGraph::new(),
            wakers: InfVec::new(),
            wake_at: None,
            source_set: USizeSet::new(),
            source_remove: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_source(&mut self, x: XKey) {
        self.source_set.insert(x.0);
    }
    fn remove_target(&mut self, y: YKey) {
        self.g.remove_y(y);
        self.wakers[y.0] = None;
    }
    fn apply_source_remove(&mut self) {
        let source_removing = self.source_remove.clone();
        let mut xs = source_removing.lock().unwrap();
        for x in xs.drain(..) {
            self.g.remove_x(x);
        }
    }

    fn wake(&mut self, x: XKey) {
        for (y, _) in self.g.ys_from_x(x) {
            wake(&mut self.wakers, y);
        }
    }
    pub fn context(&mut self) -> &mut StateContext {
        self.apply_source_remove();
        self.wake_at = None;
        StateContext::new(self)
    }
    fn commit_target<A: Into<Action>>(
        &mut self,
        waker: impl Fn() -> A,
    ) -> (Option<YKey>, Option<SpawnAtTask>) {
        let y = self.g.insert_y(());
        for x in self.source_set.iter() {
            self.g.insert_edge(XKey(x), y, ());
        }
        self.wakers[y.0] = Some(waker().into());
        let task = self.wake_at.map(|at| spawn_at(waker(), at));
        (Some(y), task)
    }
}
fn wake(wakers: &mut InfVec<Option<Action>>, y: YKey) {
    if let Some(waker) = wakers[y.0].take() {
        waker.call();
    }
}

#[derive(Ex)]
#[derive_ex(Debug)]
pub struct StateKey {
    x: XKey,
    #[debug(ignore)]
    source_remove: Arc<Mutex<Vec<XKey>>>,
}
impl StateKey {
    pub fn new(cx: &mut StateContext) -> Self {
        let x = cx.0.g.insert_x(());
        Self {
            x,
            source_remove: cx.0.source_remove.clone(),
        }
    }
    pub fn watch(&self, cx: &mut StateContext) {
        cx.0.set_source(self.x);
    }
    pub fn notify(&self, cx: &mut StateContext) {
        cx.0.wake(self.x);
    }
}
impl Drop for StateKey {
    fn drop(&mut self) {
        self.source_remove.lock().unwrap().push(self.x);
    }
}

#[repr(transparent)]
pub struct StateContext(StateGraph);

impl StateContext {
    fn new(g: &mut StateGraph) -> &mut Self {
        unsafe { transmute(g) }
    }
    pub fn notify_at(&mut self, at: Instant) {
        if let Some(wake_at) = self.0.wake_at {
            if wake_at <= at {
                return;
            }
        }
        self.0.wake_at = Some(at);
    }
}

struct RawStateContainer<St> {
    g: StateGraph,
    st: St,
}

#[derive(Ex)]
#[derive_ex(Clone, bound())]
pub struct StateContainer<St>(Arc<Mutex<RawStateContainer<St>>>);

impl<St> StateContainer<St> {
    pub fn new(f: impl FnOnce(&mut StateContext) -> St) -> Self {
        let mut g = StateGraph::new();
        let st = f(g.context());
        Self(Arc::new(Mutex::new(RawStateContainer { g, st })))
    }

    pub async fn poll_fn<U>(&self, mut f: impl FnMut(&mut St, &mut StateContext) -> Poll<U>) -> U {
        let mut t = Target::new(self);
        poll_fn(|cx| t.poll_fn(&mut f, cx)).await
    }
    pub fn poll_fn_stream<U>(
        &self,
        mut f: impl FnMut(&mut St, &mut StateContext) -> Poll<Option<U>> + 'static,
    ) -> impl Stream<Item = U> + 'static
    where
        St: 'static,
    {
        let mut t = Target::new(self);
        stream::poll_fn(move |cx| t.poll_fn(&mut f, cx))
    }
    pub fn subscribe<U>(
        &self,
        mut f: impl FnMut(&mut St, &mut StateContext) -> U + 'static,
    ) -> impl Stream<Item = U> + 'static
    where
        St: Sync + Send + 'static,
    {
        struct WatchState {
            waker: Option<Waker>,
            age: usize,
            is_dirty: bool,
        }
        let ws = WatchState {
            waker: None,
            age: 0,
            is_dirty: true,
        };
        fn wake(ws: Arc<Mutex<WatchState>>, age: usize) {
            let mut ws = ws.lock().unwrap();
            if ws.age == age {
                ws.is_dirty = true;
                let waker = ws.waker.take();
                if let Some(waker) = waker {
                    drop(ws);
                    waker.wake();
                }
            }
        }

        let ws_arc = Arc::new(Mutex::new(ws));
        let mut t = Target::new(self);
        stream::poll_fn(move |cx| {
            let st = &mut *t.st.0.lock().unwrap();
            let mut ws = ws_arc.lock().unwrap();
            if ws.is_dirty {
                if let Some(key) = t.key {
                    st.g.remove_target(key);
                    t.key = None;
                }
                st.g.source_set.clear();
                t.sleep = None;
                ws.age = ws.age.wrapping_add(1);
                ws.is_dirty = false;
                let value = f(&mut st.st, st.g.context());
                (t.key, t.sleep) =
                    st.g.commit_target(|| Action::from_arc_fn_usize(ws_arc.clone(), wake, ws.age));
                Poll::Ready(Some(value))
            } else {
                ws.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        })
    }

    pub fn update<T>(&self, f: impl FnOnce(&mut St, &mut StateContext) -> T) -> T {
        let ss = &mut *self.0.lock().unwrap();
        f(&mut ss.st, ss.g.context())
    }
}

struct Target<St> {
    key: Option<YKey>,
    st: StateContainer<St>,
    sleep: Option<SpawnAtTask>,
}
impl<St> Target<St> {
    pub fn new(st: &StateContainer<St>) -> Self {
        Self {
            key: None,
            st: st.clone(),
            sleep: None,
        }
    }
    fn poll_fn<T>(
        &mut self,
        mut f: impl FnMut(&mut St, &mut StateContext) -> Poll<T>,
        cx: &mut Context,
    ) -> Poll<T> {
        let st = &mut *self.st.0.lock().unwrap();
        if let Some(y) = self.key.take() {
            st.g.remove_target(y);
        }
        st.g.source_set.clear();
        self.sleep = None;
        match f(&mut st.st, st.g.context()) {
            Poll::Ready(value) => Poll::Ready(value),
            Poll::Pending => {
                (self.key, self.sleep) = st.g.commit_target(|| cx.waker());
                Poll::Pending
            }
        }
    }
}

impl<St> Drop for Target<St> {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.st.0.lock().unwrap().g.remove_target(key);
        }
    }
}
