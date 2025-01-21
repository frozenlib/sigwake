use std::{collections::VecDeque, task::Poll};

use futures::Stream;

use crate::{
    StateContainer, StateContext, StateKey,
    utils::shared_queue::{SharedQueue, SharedQueueCursor},
};

pub struct EventChannel<T> {
    queue: SharedQueue<T>,
    key: StateKey,
}

impl<T> EventChannel<T> {
    pub fn new(cx: &mut StateContext) -> Self {
        Self {
            queue: SharedQueue::new(),
            key: StateKey::new(cx),
        }
    }
    pub fn send(&mut self, value: T, cx: &mut StateContext) {
        self.queue.push(value);
        self.key.notify(cx);
    }
    pub fn send_all(&mut self, values: impl IntoIterator<Item = T>, cx: &mut StateContext) {
        self.queue.extend(values);
        self.key.notify(cx);
    }
}

impl<St> StateContainer<St> {
    pub fn subscribe_event<T: Clone + 'static>(
        &self,
        channel: impl Fn(&mut St) -> &mut EventChannel<T> + 'static,
    ) -> impl Stream<Item = T> + 'static
    where
        St: 'static,
    {
        self.subscribe_event_with(channel, |_st, _cx| [], |e| Some(e.clone()))
    }
    pub fn subscribe_event_with<T, U, I>(
        &self,
        channel: impl Fn(&mut St) -> &mut EventChannel<T> + 'static,
        inits: impl FnOnce(&mut St, &mut StateContext) -> I + 'static,
        mut filter_map: impl FnMut(&T) -> Option<U> + 'static,
    ) -> impl Stream<Item = U> + 'static
    where
        St: 'static,
        T: 'static,
        U: 'static,
        I: IntoIterator<Item = U>,
    {
        let (mut items, cursor) = self.update(|st, cx| {
            (
                inits(st, cx).into_iter().collect::<VecDeque<_>>(),
                channel(st).queue.create_cursor(),
            )
        });
        let mut s = Scope {
            st: self.clone(),
            channel,
            cursor: Some(cursor),
        };
        self.poll_fn_stream(move |st, cx| {
            if items.is_empty() {
                items.extend(
                    (s.channel)(st)
                        .queue
                        .read(s.cursor.as_mut().unwrap())
                        .iter()
                        .filter_map(&mut filter_map),
                );
            }
            if let Some(item) = items.pop_front() {
                Poll::Ready(Some(item))
            } else {
                (s.channel)(st).key.watch(cx);
                Poll::Pending
            }
        })
    }
}
struct Scope<St, T, ToChannel>
where
    ToChannel: Fn(&mut St) -> &mut EventChannel<T>,
{
    st: StateContainer<St>,
    channel: ToChannel,
    cursor: Option<SharedQueueCursor<T>>,
}
impl<St, T, ToChannel> Drop for Scope<St, T, ToChannel>
where
    ToChannel: Fn(&mut St) -> &mut EventChannel<T>,
{
    fn drop(&mut self) {
        if let Some(cursor) = self.cursor.take() {
            self.st.update(|st, _cx| {
                (self.channel)(st).queue.drop_cursor(cursor);
            });
        }
    }
}
