use sentinel_io::runtime::Runtime;
use std::{future::Future, task::{Context, Poll, Waker}};
use std::time::{Instant, Duration};
use std::thread;

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
    start: Instant,
    duration: Duration
}

impl Timer {
    fn new(duration: Duration) -> Self {
        Self { start: Instant::now(), duration }
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if self.start.elapsed() < self.duration {
            return Poll::Pending;
        } else {
            println!("Timer finish");
            return Poll::Ready(());
        }
    }
}

fn delay_wake(waker: Waker) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {}
    waker.wake();
}

fn main() {

    let c1 = Box::pin(Counter::new(1, 5));
    let c2 = Box::pin(Counter::new(2, 7));
    let c3 = Box::pin(Counter::new(3, 10));
    let t1 = Box::pin(Timer::new(Duration::from_secs(5)));
    let t2 = Box::pin(Timer::new(Duration::from_secs(3)));

    let mut runtime = Runtime::new();
    runtime.push(c1, false);
    runtime.push(t1, true);
    runtime.push(c2, false);
    runtime.push(c3, false);
    runtime.push(t2, false);

    // thread::spawn(move || {
    //     delay_wake(waker.clone());
    // });

    runtime.run();
}
