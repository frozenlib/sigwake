use std::time::Duration;

use sigwake::state::QueueReader;
use tokio::time::sleep;
use tokio::{spawn, test};

use sigwake::{StateContainer, state::Queue};

#[test]
async fn push_then_pop() {
    let st = StateContainer::new(Queue::new);

    st.update(|st, cx| {
        st.push(42, cx);
    });
    let ret = st.poll_fn(|st, cx| st.pop(cx)).await;

    assert_eq!(ret, 42);
}

#[test]
async fn push_many_then_pop() {
    let st = StateContainer::new(Queue::new);
    st.update(|st, cx| {
        st.push(42, cx);
        st.push(43, cx);
    });
    let ret = st.poll_fn(|st, cx| st.pop(cx)).await;
    assert_eq!(ret, 42);

    let ret = st.poll_fn(|st, cx| st.pop(cx)).await;
    assert_eq!(ret, 43);
}

#[test]
async fn pop_then_push() {
    let st = StateContainer::new(Queue::new);
    spawn({
        let st = st.clone();
        async move {
            sleep(Duration::from_millis(100)).await;
            st.update(|st, cx| {
                st.push(42, cx);
            });
        }
    });
    let ret = st.poll_fn(|st, cx| st.pop(cx)).await;
    assert_eq!(ret, 42);
}

#[test]
async fn push_then_fetch() {
    let st = StateContainer::new(Queue::new);
    st.update(|st, cx| {
        st.push(42, cx);
    });
    let mut reader = QueueReader::new();
    st.poll_fn(|st, cx| reader.fetch(st, cx)).await;
    let ret = reader.into_iter().collect::<Vec<_>>();
    assert_eq!(ret, vec![42]);
}

#[test]
async fn push_many_then_fetch() {
    let st = StateContainer::new(Queue::new);
    st.update(|st, cx| {
        st.push(42, cx);
        st.push(43, cx);
    });
    let mut reader = QueueReader::new();
    st.poll_fn(|st, cx| reader.fetch(st, cx)).await;
    let ret = reader.into_iter().collect::<Vec<_>>();
    assert_eq!(ret, vec![42, 43]);
}

#[test]
async fn fetch_then_push() {
    let st = StateContainer::new(Queue::new);
    spawn({
        let st = st.clone();
        async move {
            sleep(Duration::from_millis(100)).await;
            st.update(|st, cx| {
                st.push(42, cx);
            });
        }
    });

    let mut reader = QueueReader::new();
    st.poll_fn(|st, cx| reader.fetch(st, cx)).await;
    let ret = reader.into_iter().collect::<Vec<_>>();
    assert_eq!(ret, vec![42]);
}
