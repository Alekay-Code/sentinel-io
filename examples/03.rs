use sentinel_io as sentinel;
use std::{future::Future, task::Poll};
use sentinel_io::time::Timer;
use std::time::Duration;
use sentinel_io::runtime;

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
        println!("[{}] counter: {}", self.id, self.current);
        self.current += 1;

        if self.current >= self.to {
            println!("[{}] counter finish: {}", self.id, self.current);
            return Poll::Ready(self.current);
        } else {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
    }
}

#[sentinel::main]
async fn main() {
    let c1 = Counter::new(1,5);

    let h2 = sentinel::spawn(async {
        let c2 = Counter::new(2, 10);
        c2.await;
    });

    let h3 = sentinel::spawn(async {
        let timer = Timer::new(Duration::from_secs(5));
        timer.await;
    });

    c1.await;
    h2.await;
    h3.await;
}
