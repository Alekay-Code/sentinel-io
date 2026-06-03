use sentinel_io as sentinel;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

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

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("[C{}] counter: {}", self.id, self.current);
        self.current += 1;

        if self.current >= self.to {
            return Poll::Ready(self.current);
        } else {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
    }
}

fn main() {
    sentinel::block_on(async {
        let workers = 10000;
        let count   = 2;

        let handles: Vec<_> = (0..workers)
            .map(|id| sentinel::spawn(Counter::new(id, count)))
            .collect();

        for handle in handles {
            let result = handle.await;
            println!("worker finished: {result}");
        }
    });
}
