# sigwake

[![Crates.io](https://img.shields.io/crates/v/sigwake.svg)](https://crates.io/crates/sigwake)
[![Docs.rs](https://docs.rs/sigwake/badge.svg)](https://docs.rs/sigwake/)
[![Actions Status](https://github.com/frozenlib/sigwake/workflows/CI/badge.svg)](https://github.com/frozenlib/sigwake/actions)

マルチスレッドで使用できるシグナル型の状態管理ライブラリ

sigwake は近年の Web 用フレームワークの多くで採用されているシグナルを用いた状態管理を Rust の非同期プログラミングモデルによって再解釈したライブラリです。

## 特徴

- 状態へのアクセスから依存関係を記録する Signal 型のリアクティブプログラミングをサポート
- Rust の 非同期プログラミングモデル (`async/await`, `Future`, `Waker`) との統合
- マルチスレッド対応
- 任意の非同期ランタイムと組み合わせて使用できる、非同期ランタイムに依存しない実装
- コンパクトな API

## 例

```rust
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
```

Output:

```txt
1
4
```

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
