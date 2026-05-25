use sentinel_io::runtime::{self, Runtime};
use std::{future::Future, task::Poll};
use std::time::Duration;
use sentinel_io::time::Timer;

struct Counter {
    id: i32,
    current: usize,
    to: usize
}

impl Counter {
    fn new(id: i32, to: usize) -> Self {
        Self {id, current: 0, to }
    }
}

impl Future for Counter {
    type Output = usize;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
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

fn main() {
    let mut runtime = Runtime::new();
    let c1 = Box::pin(Counter::new(1, 5));
    let c2 = Box::pin(Counter::new(2, 7));
    // let c3 = Box::pin(Counter::new(3, 10));
    let t1 = Box::pin(Timer::new(Duration::from_secs(5)));
    // let t2 = Box::pin(Timer::new(Duration::from_secs(20)));

    runtime::block_on(async move {

        // runtime.run();

        // INFO: Block until task complete
        let h1 = runtime.spawn(async move {
            let x       = c1.await;
            let elapsed = t1.await;
            println!("Timer elapsed: {}", elapsed.as_nanos());
            return x;
        });

        runtime.spawn(async move {
            c2.await;
        });

        runtime.run();

        let res = h1.await;
        println!("RES: {res}");
    });
}
