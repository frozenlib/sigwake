use std::{
    task::Poll,
    time::{Duration, Instant},
};

use assert_call::{Call, CallRecorder, call};
use futures::StreamExt;
use sigwake::{StateContainer, StateKey, state::Value};
use tokio::{spawn, test, time::sleep};

#[derive(Clone)]
struct Ss(StateContainer<St>);

impl Ss {
    fn new() -> Self {
        Self(St::new())
    }
    fn set_a(&self, a: u32) {
        self.0.update(|st, cx| {
            *st.a.get_mut(cx) = a;
        });
    }
    fn set_b(&self, b: u32) {
        self.0.update(|st, cx| {
            *st.b.get_mut(cx) = b;
        });
    }

    async fn wait_ab_10(&self) -> u32 {
        self.0
            .poll_fn(|st, cx| {
                let a = *st.a.get(cx);
                let b = *st.b.get(cx);
                if a + b >= 10 {
                    Poll::Ready(a + b)
                } else {
                    Poll::Pending
                }
            })
            .await
    }
}

struct St {
    a: Value<u32>,
    b: Value<u32>,
}
impl St {
    fn new() -> StateContainer<Self> {
        StateContainer::new(|cx| Self {
            a: Value::new(0, cx),
            b: Value::new(0, cx),
        })
    }
}

#[test]
async fn wait_for_over_10() -> anyhow::Result<()> {
    let mut cr = CallRecorder::new();

    let ss = Ss::new();
    let task = spawn({
        let ss = ss.clone();
        async move {
            let ab = ss.wait_ab_10().await;
            call!("ready {ab}");
        }
    });
    sleep(Duration::from_millis(100)).await;
    ss.set_a(5);
    sleep(Duration::from_millis(100)).await;
    ss.set_b(6);
    task.await?;
    cr.verify("ready 11");

    Ok(())
}

#[test]
async fn wait_parallel() -> anyhow::Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();
    let task0 = spawn({
        let ss = ss.clone();
        async move {
            let ab = ss.wait_ab_10().await;
            call!("ready 0 {ab}");
        }
    });
    let task1 = spawn({
        let ss = ss.clone();
        async move {
            let ab = ss.wait_ab_10().await;
            call!("ready 1 {ab}");
        }
    });
    sleep(Duration::from_millis(100)).await;
    ss.set_a(5);
    sleep(Duration::from_millis(100)).await;
    ss.set_b(6);
    task0.await?;
    task1.await?;
    cr.verify(Call::par(["ready 0 11", "ready 1 11"]));

    Ok(())
}

#[test]
async fn notify_at() -> anyhow::Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();
    let end = Instant::now() + Duration::from_millis(1000);
    let task = spawn({
        async move {
            ss.0.poll_fn(|_st, cx| {
                cx.notify_at(end);
                if Instant::now() >= end {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            call!("ready");
        }
    });
    cr.verify(());
    sleep(Duration::from_millis(100)).await;
    cr.verify(());
    task.await?;
    cr.verify("ready");
    Ok(())
}

#[test]
async fn notify_at_2_a() -> anyhow::Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();
    let end1 = Instant::now() + Duration::from_millis(1000);
    let end2 = Instant::now() + Duration::from_millis(5000);
    let _task = spawn({
        async move {
            ss.0.poll_fn(|_st, cx| {
                cx.notify_at(end1);
                cx.notify_at(end2);
                if Instant::now() >= end1 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            call!("ready");
        }
    });
    cr.verify(());
    sleep(Duration::from_millis(2000)).await;
    cr.verify("ready");
    Ok(())
}

#[test]
async fn notify_at_2_b() -> anyhow::Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();
    let end1 = Instant::now() + Duration::from_millis(1000);
    let end2 = Instant::now() + Duration::from_millis(5000);
    let _task = spawn({
        async move {
            ss.0.poll_fn(|_st, cx| {
                cx.notify_at(end2);
                cx.notify_at(end1);
                if Instant::now() >= end1 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            call!("ready");
        }
    });
    cr.verify(());
    sleep(Duration::from_millis(2000)).await;
    cr.verify("ready");
    Ok(())
}

#[test]
async fn subscribe() -> anyhow::Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();

    let mut stream = ss.0.subscribe(|st, cx| {
        let a = *st.a.get(cx);
        let b = *st.b.get(cx);
        a + b
    });
    spawn(async move {
        while let Some(sum) = stream.next().await {
            call!("sum {sum}");
        }
    });

    sleep(Duration::from_millis(100)).await;
    cr.verify("sum 0");

    ss.set_a(3);
    sleep(Duration::from_millis(100)).await;
    cr.verify("sum 3");

    ss.set_b(4);
    sleep(Duration::from_millis(100)).await;
    cr.verify("sum 7");

    Ok(())
}

#[test]
async fn subscribe_multiple() -> anyhow::Result<()> {
    let mut cr = CallRecorder::new();
    let ss = Ss::new();

    let mut s1 = ss.0.subscribe(|st, cx| *st.a.get(cx));
    spawn(async move {
        while let Some(a) = s1.next().await {
            call!("a {a}");
        }
    });

    let mut s2 = ss.0.subscribe(|st, cx| *st.b.get(cx));
    spawn(async move {
        while let Some(b) = s2.next().await {
            call!("b {b}");
        }
    });

    sleep(Duration::from_millis(100)).await;
    cr.verify(Call::par(["a 0", "b 0"]));

    ss.set_a(3);
    sleep(Duration::from_millis(100)).await;
    cr.verify("a 3");

    ss.set_b(4);
    sleep(Duration::from_millis(100)).await;
    cr.verify("b 4");

    Ok(())
}

#[test]
async fn dependency_with_unused_source() {
    struct St {
        _unused: StateKey,
        value: Value<u32>,
    }
    impl St {
        fn new() -> StateContainer<Self> {
            StateContainer::new(|cx| Self {
                _unused: StateKey::new(cx),
                value: Value::new(0, cx),
            })
        }
    }
    let mut cr = CallRecorder::new();
    let st = St::new();
    let mut s = st.subscribe(|st, cx| *st.value.get(cx));
    spawn(async move {
        while let Some(value) = s.next().await {
            call!("{value}");
        }
    });
    for i in 1..=3 {
        sleep(Duration::from_millis(10)).await;
        st.update(|st, cx| {
            *st.value.get_mut(cx) = i;
        });
    }
    sleep(Duration::from_millis(10)).await;
    cr.verify(["0", "1", "2", "3"]);
}

#[test]
async fn dependency_with_unused_target() {
    struct St {
        value: Value<u32>,
    }
    impl St {
        fn new() -> StateContainer<Self> {
            StateContainer::new(|cx| Self {
                value: Value::new(0, cx),
            })
        }
    }

    let mut cr = CallRecorder::new();
    let st = St::new();
    spawn({
        let st = st.clone();
        async move {
            st.poll_fn(|_st, _cx| Poll::<()>::Pending).await;
        }
    });
    sleep(Duration::from_millis(50)).await;
    let mut s = st.subscribe(|st, cx| *st.value.get(cx));
    spawn(async move {
        while let Some(value) = s.next().await {
            call!("{value}");
        }
    });
    for i in 1..=3 {
        sleep(Duration::from_millis(10)).await;
        st.update(|st, cx| {
            *st.value.get_mut(cx) = i;
        });
    }
    sleep(Duration::from_millis(10)).await;
    cr.verify(["0", "1", "2", "3"]);
}

#[test]
async fn test_untracked() {
    struct St {
        value: Value<u32>,
    }
    impl St {
        fn new() -> StateContainer<Self> {
            StateContainer::new(|cx| Self {
                value: Value::new(0, cx),
            })
        }
    }
    let st = St::new();
    assert_eq!(*st.lock_untracked().value.get_untracked(), 0);
    st.update(|st, cx| {
        st.value.set(20, cx);
    });
    assert_eq!(*st.lock_untracked().value.get_untracked(), 20);
}
