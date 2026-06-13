# Sentinel-IO

Sentinel-IO is an asynchronous runtime written in Rust for building I/O-backed
applications. It provides its own executor, task scheduler, async networking
primitives, and a procedural macro for bootstrapping async programs.

## Features

| | |
|---|---|
| 🚀 **Executor & Scheduler** | A custom single-threaded, round-robin task scheduler powering `spawn()` and `block_on()`, with `JoinHandle<T>` to await spawned tasks. |
| 🌐 **Async Networking** | `TcpListener` and `TcpStream` with async `accept`, `read` and `write`, plus `split()` to turn a stream into an independent reader and writer. |
| ⏱️ **Timers** | A `Timer` future for delaying execution without blocking the runtime. |
| 🔌 **Async I/O Traits** | `AsyncRead` and `AsyncWrite`, implemented for the provided networking types. |
| ✨ **`#[sentinel::main]`** | A procedural macro that turns `async fn main()` into a runnable entry point. |

## Example

```rust
use sentinel_io as sentinel;
use sentinel::io::{AsyncRead, AsyncWrite};
use sentinel::net::{TcpListener, TcpStream};
use std::net::SocketAddr;

const SERVER_ADDR: &str = "127.0.0.1:8000";

async fn handle_connection(mut conn: TcpStream, addr: SocketAddr) {
    let mut buffer = [0u8; 1024];

    loop {
        let n = conn.read(&mut buffer).await.unwrap();
        if n == 0 {
            return;
        }
        let _ = conn.write(&buffer[0..n]).await;
    }
}

#[sentinel::main]
async fn main() {
    let mut server = TcpListener::bind(SERVER_ADDR).unwrap();

    loop {
        let (client, addr) = server.accept().await.unwrap();
        sentinel::spawn(handle_connection(client, addr));
    }
}
```

More examples are available in the [`examples/`](examples) directory.

## Roadmap

The following objectives are still pending:

- [ ] I/O Driver (event-driven I/O multiplexing via epoll/kqueue, replacing
      busy-polling)
- [ ] Multi-threading executor
- [ ] Async file I/O
- [ ] Timeout support for I/O operations
- [ ] UDP support
- [ ] Improved error handling

## License

This project is licensed under the [MIT License](LICENSE).
