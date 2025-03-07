# sigwake

[![Crates.io](https://img.shields.io/crates/v/sigwake.svg)](https://crates.io/crates/sigwake)
[![Docs.rs](https://docs.rs/sigwake/badge.svg)](https://docs.rs/sigwake/)
[![Actions Status](https://github.com/frozenlib/sigwake/workflows/CI/badge.svg)](https://github.com/frozenlib/sigwake/actions)

A thread-safe state management library using signals

sigwake is a library that reinterprets signal-based state management (commonly used in modern web frameworks) through Rust's asynchronous programming model.

## Features

- Supports reactive programming with Signal-based dependency tracking from state access
- Integration with Rust's asynchronous programming model (`async/await`, `Future`, `Waker`)
- Thread-safe implementation
- Runtime-agnostic implementation that can be used with any async runtime
- Compact API

## Example

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

    // Stream that emits values whenever dependent values are updated
    let mut ac = st.subscribe(|st, cx| st.a.get(cx) + st.c.get(cx));
    spawn(async move {
        while let Some(value) = ac.next().await {
            println!("{value}");
        }
    });
    st.update(|st, cx| st.a.set(1, cx)); // Update a
    sleep(Duration::from_secs(1)).await;

    st.update(|st, cx| st.b.set(2, cx)); // Update b (ac is not recalculated)
    sleep(Duration::from_secs(1)).await;

    st.update(|st, cx| st.c.set(3, cx)); // Update c
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
