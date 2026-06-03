use std::time::Duration;

use sentinel_io::time::Timer;
use sentinel_io as sentinel;
use sentinel::net::{TcpListener, TcpStream};
use sentinel::io::{AsyncRead, AsyncWrite};

use std::{future::Future, task::Poll};

struct Counter {
    id: i32,
    current: usize,
    to: usize,
}

impl Counter {
    fn new(id: i32, to: usize) -> Self {
        Self { id, current: 0, to }
    }
}

impl Future for Counter {
    type Output = usize;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        println!("[C{}] count: {}", self.id, self.current);
        self.current += 1;

        if self.current >= self.to {
            return Poll::Ready(self.current);
        } else {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
    }
}

const SERVER_ADDR: &str = "localhost:8000";
const CLIENT_ADDR: &str = "localhost:8000";

fn main() {
    sentinel::block_on(async {
        let server = sentinel::spawn(async {
            let mut buffer = [0u8; 1024];
            let mut server = TcpListener::bind(SERVER_ADDR).unwrap();
            println!("[SERVER]: Waiting for connections at {SERVER_ADDR}");
            let (mut client, addr) = server.accept().await.unwrap();
            let addr = addr.to_string();
            println!("[SERVER]: New connection {}", addr);
            let msg = "hello from async server";
            println!("[SERVER]: Sending '{msg}' to {}", addr);
            let _ = client.write(msg.as_bytes()).await;
            let amount = client.read(&mut buffer).await.unwrap();
            let msg = String::from_utf8_lossy(&buffer[0..amount]);
            println!("[SERVER]: Read {amount} bytes: '{msg}'");
        });

        let client = sentinel::spawn(async {
            let mut client = TcpStream::connect(CLIENT_ADDR).unwrap();
            let mut buffer = [0u8; 1024];
            let amount = client.read(&mut buffer).await.unwrap();
            let msg = String::from_utf8_lossy(&buffer[0..amount]);
            println!("[CLIENT]: Read {amount} bytes: '{msg}'");
            println!("[CLIENT]: Waiting 3 seconds...");
            Timer::new(Duration::from_secs(3)).await;
            println!("[CLIENT]: Sending '{msg}' back");
            let _ = client.write(&buffer[0..amount]).await;
        });

        let counter = sentinel::spawn(async {
            println!("[COUNTER]: start...");
            let counter = Counter::new(1, 20);
            let count = counter.await;
            println!("[COUNTER]: finish counting to {count}");
        });

        server.await;
        client.await;
        counter.await;
    });

}
