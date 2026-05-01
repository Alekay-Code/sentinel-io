use sentinel_io::runtime::Runtime;
use std::{future::Future, task::Poll};
use std::time::{Instant, Duration};

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
    type Output = ();

    fn poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        println!("[{}] counter: {}", self.id, self.current);
        self.current += 1;

        if self.current >= self.to {
            println!("[{}] counter finish: {}", self.id, self.current);
            return Poll::Ready(());
        } else {
            return Poll::Pending;
        }
    }
}

struct Timer {
    id: usize,
    start: Instant,
    duration: Duration
}

impl Timer {
    fn new(id: usize, duration: Duration) -> Self {
        Self { id, start: Instant::now(), duration }
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if self.start.elapsed() < self.duration {
            println!("Timer[{}] poll -> {}s", self.id, self.start.elapsed().as_secs());
            return Poll::Pending;
        } else {
            println!("Timer[{}] finish -> {}/{}s", self.id, self.duration.as_secs(), self.start.elapsed().as_secs());
            return Poll::Ready(());
        }
    }
}

fn main() {

    let c1 = Box::pin(Counter::new(1, 5));
    let c2 = Box::pin(Counter::new(2, 7));
    let c3 = Box::pin(Counter::new(3, 10));
    let t1 = Box::pin(Timer::new(1, Duration::from_secs(5)));
    let t2 = Box::pin(Timer::new(2, Duration::from_secs(20)));

    let mut runtime = Runtime::new();
    runtime.push(c1, false);
    runtime.push(t1, true);
    runtime.push(c2, false);
    runtime.push(c3, false);
    runtime.push(t2, true);

    runtime.run();
}
