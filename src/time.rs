use std::{
    future::poll_fn,
    sync::{Condvar, LazyLock, Mutex},
    task::Poll,
    thread::spawn,
    time::{Duration, Instant, SystemTime},
};

use crate::utils::Action;
use crate::utils::btree_multi_map::BTreeMultiMap;

#[derive(Debug)]
pub struct SpawnAtTask {
    at: RawAnyTime,
    id: usize,
}
impl Drop for SpawnAtTask {
    fn drop(&mut self) {
        TIMER.cancel(self);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct AnyTime(RawAnyTime);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum RawAnyTime {
    Instant(Instant),
    SystemTime(SystemTime),
}
impl RawAnyTime {
    fn is_ready(&self) -> bool {
        match self {
            RawAnyTime::Instant(instant) => Instant::now() >= *instant,
            RawAnyTime::SystemTime(system_time) => SystemTime::now() >= *system_time,
        }
    }
}

impl From<Instant> for AnyTime {
    fn from(value: Instant) -> Self {
        Self(RawAnyTime::Instant(value))
    }
}
impl From<SystemTime> for AnyTime {
    fn from(value: SystemTime) -> Self {
        Self(RawAnyTime::SystemTime(value))
    }
}
impl From<Duration> for AnyTime {
    fn from(value: Duration) -> Self {
        Self(RawAnyTime::Instant(Instant::now() + value))
    }
}

pub fn spawn_at(action: impl Into<Action>, at: impl Into<AnyTime>) -> SpawnAtTask {
    spawn_at_raw(action.into(), at.into())
}
fn spawn_at_raw(action: Action, at: AnyTime) -> SpawnAtTask {
    TIMER.spawn_at(action, at.0)
}

pub async fn sleep(time: impl Into<AnyTime>) {
    let time = time.into();
    let mut _task = None;
    poll_fn(|cx| {
        if time.0.is_ready() {
            Poll::Ready(())
        } else {
            _task = Some(spawn_at_raw(Action::from(cx.waker().clone()), time));
            Poll::Pending
        }
    })
    .await
}

static TIMER: LazyLock<Timer> = LazyLock::new(Timer::new);
static THREAD_CACHE_DURATION: Duration = Duration::from_secs(4);

struct TimerData {
    actions_instant: BTreeMultiMap<Instant, Action>,
    actions_system_time: BTreeMultiMap<SystemTime, Action>,
    is_running: bool,
}
impl TimerData {
    const fn new() -> Self {
        Self {
            actions_instant: BTreeMultiMap::new(),
            actions_system_time: BTreeMultiMap::new(),
            is_running: false,
        }
    }

    fn step(&mut self) -> TimerStep {
        let mut next = Duration::MAX;
        if let Some(e) = self.actions_instant.first_entry() {
            let now = Instant::now();
            let key = e.key().0;
            if now >= key {
                return TimerStep::Call(e.remove());
            }
            next = key - now;
        }
        if let Some(e) = self.actions_system_time.first_entry() {
            let now = SystemTime::now();
            let key = e.key().0;
            if now >= key {
                return TimerStep::Call(e.remove());
            }
            if let Ok(next_new) = key.duration_since(now) {
                next = next.min(next_new);
            }
            next = next.min(Duration::from_secs(1));
        }
        if next == Duration::MAX {
            TimerStep::None
        } else {
            TimerStep::Wait(next)
        }
    }
}

enum TimerStep {
    None,
    Call(Action),
    Wait(Duration),
}

struct Timer {
    data: Mutex<TimerData>,
    cvar: Condvar,
}
impl Timer {
    const fn new() -> Self {
        Self {
            data: Mutex::new(TimerData::new()),
            cvar: Condvar::new(),
        }
    }
}
impl Timer {
    fn spawn_at(&self, action: Action, at: RawAnyTime) -> SpawnAtTask {
        let mut data = self.data.lock().unwrap();
        let is_wake;
        let id;
        match at {
            RawAnyTime::Instant(at) => {
                is_wake = is_wake_with(&data.actions_instant, at);
                id = data.actions_instant.insert(at, action);
            }
            RawAnyTime::SystemTime(at) => {
                is_wake = is_wake_with(&data.actions_system_time, at);
                id = data.actions_system_time.insert(at, action);
            }
        }
        if is_wake {
            if data.is_running {
                self.cvar.notify_one();
            } else {
                data.is_running = true;
                spawn(|| TIMER.run());
            }
        }
        SpawnAtTask { at, id }
    }
    fn cancel(&self, task: &SpawnAtTask) {
        let mut data = self.data.lock().unwrap();
        match task.at {
            RawAnyTime::Instant(at) => {
                data.actions_instant.remove(at, task.id);
            }
            RawAnyTime::SystemTime(at) => {
                data.actions_system_time.remove(at, task.id);
            }
        }
        if data.actions_instant.is_empty() && data.actions_system_time.is_empty() {
            self.cvar.notify_one();
        }
    }
    fn run(&self) {
        let mut used = false;
        let mut last_used = Instant::now();
        loop {
            let mut data = self.data.lock().unwrap();
            match data.step() {
                TimerStep::None => {
                    let now = Instant::now();
                    if used {
                        last_used = now;
                    } else if now.duration_since(last_used) >= THREAD_CACHE_DURATION {
                        data.is_running = false;
                        return;
                    }
                    used = self.cvar.wait_timeout(data, THREAD_CACHE_DURATION).is_ok();
                }
                TimerStep::Call(action) => {
                    drop(data);
                    action.call();
                    used = true;
                }
                TimerStep::Wait(dur) => {
                    used |= self.cvar.wait_timeout(data, dur).is_ok();
                }
            }
        }
    }
}

fn is_wake_with<K: Ord + Copy, V>(actions: &BTreeMultiMap<K, V>, at: K) -> bool {
    if let Some(key) = actions.first_key() {
        at < *key
    } else {
        true
    }
}
