use std::time::Duration;

use anyhow::Result;
use assert_call::{Call, CallRecorder, call};
use futures::StreamExt;
use sigwake::{StateContainer, state::EventChannel};
use tokio::{spawn, test, time::sleep};

#[derive(Clone)]
struct Ss(StateContainer<St>);

impl Ss {
    fn new() -> Self {
        Self(St::new())
    }

    fn send_event(&self, value: u32) {
        self.0.update(|st, cx| {
            st.e.send(value, cx);
        });
    }
}

struct St {
    e: EventChannel<u32>,
}
impl St {
    fn new() -> StateContainer<Self> {
        StateContainer::new(|cx| Self {
            e: EventChannel::new(cx),
        })
    }
}

async fn wait_sleep() {
    sleep(Duration::from_millis(1000)).await;
}

#[test]
async fn test_subscribe_event() -> Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();
    let mut es = ss.0.subscribe_event(|st| &mut st.e);
    spawn(async move {
        while let Some(e) = es.next().await {
            call!("{e}");
        }
    });
    ss.0.update(|st, cx| {
        st.e.send(1, cx);
        st.e.send(2, cx);
    });
    wait_sleep().await;
    cr.verify(vec!["1", "2"]);
    Ok(())
}

#[test]
async fn test_subscribe_event_reader2() -> Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();
    let mut es_a = ss.0.subscribe_event(|st| &mut st.e);
    spawn(async move {
        while let Some(e) = es_a.next().await {
            call!("a{e}");
        }
    });
    let mut es_b = ss.0.subscribe_event(|st| &mut st.e);
    spawn(async move {
        while let Some(e) = es_b.next().await {
            call!("b{e}");
        }
    });
    ss.0.update(|st, cx| {
        st.e.send(1, cx);
        st.e.send(2, cx);
    });
    wait_sleep().await;
    cr.verify(Call::par([&["a1", "a2"], &["b1", "b2"]]));
    Ok(())
}

#[test]
async fn test_late_reader() -> Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();

    ss.0.update(|st, cx| {
        st.e.send(1, cx);
        st.e.send(2, cx);
    });

    let mut es = ss.0.subscribe_event(|st| &mut st.e);

    ss.0.update(|st, cx| {
        st.e.send(3, cx);
    });

    spawn(async move {
        while let Some(e) = es.next().await {
            call!("late{e}");
        }
    });

    wait_sleep().await;
    cr.verify(vec!["late3"]);
    Ok(())
}

#[test]
async fn subscribe_event_with_test() -> anyhow::Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();

    let mut stream = ss.0.subscribe_event_with(
        |st| &mut st.e,
        |_st, _cx| [100],
        |e| if e % 2 == 0 { Some(e * 10) } else { None },
    );

    spawn(async move {
        while let Some(e) = stream.next().await {
            call!("event {e}");
        }
    });

    sleep(Duration::from_millis(100)).await;
    cr.verify("event 100");

    ss.send_event(1);
    ss.send_event(2);
    ss.send_event(3);
    ss.send_event(4);

    sleep(Duration::from_millis(100)).await;
    cr.verify(vec!["event 20", "event 40"]);

    Ok(())
}
