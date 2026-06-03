use sentinel_io as sentinel;
use std::{future::Future, task::Poll};
use sentinel_io::time::Timer;
use std::time::Duration;

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

#[sentinel::main]
async fn main() {
    let id = 1;
    let c1 = Counter::new(id ,5);

    let h2 = sentinel::spawn(async {
        let id = 2;
        let c2 = Counter::new(id, 10);
        c2.await;
        println!("[C{id}] Counter finish");
    });

    let h3 = sentinel::spawn(async {
        let dur = Duration::from_secs(5);
        println!("[T1] Timer set to {}s", dur.as_secs());
        let timer = Timer::new(dur);
        let elapsed = timer.await;
        println!("[T1] Timer elapsed {}ms", elapsed.as_millis());
    });

    c1.await;
    println!("[C{id}] Counter finish");

    h2.await;
    h3.await;
}
