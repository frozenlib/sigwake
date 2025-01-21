use std::time::Duration;

use futures::StreamExt;
use sigwake::{StateContainer, state::Value};
use tokio::{spawn, time::sleep};

struct State {
    a: Value<i32>,
    b: Value<i32>,
    c: Value<i32>,
}

#[tokio::main]
async fn main() {
    let st = StateContainer::new(|cx| State {
        a: Value::new(0, cx),
        b: Value::new(0, cx),
        c: Value::new(0, cx),
    });

    // 依存する値が更新される度に値が流れるストリーム
    let mut ac = st.subscribe(|st, cx| st.a.get(cx) + st.c.get(cx));
    spawn(async move {
        while let Some(value) = ac.next().await {
            println!("{value}");
        }
    });
    st.update(|st, cx| st.a.set(1, cx)); // aを更新
    sleep(Duration::from_secs(1)).await;

    st.update(|st, cx| st.b.set(2, cx)); // bを更新 (acは再計算されない)
    sleep(Duration::from_secs(1)).await;

    st.update(|st, cx| st.c.set(3, cx)); // cを更新
    sleep(Duration::from_secs(1)).await;
}
