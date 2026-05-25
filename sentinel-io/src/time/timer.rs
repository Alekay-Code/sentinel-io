use std::{future::Future, task::Poll};
use std::time::{Instant, Duration};

pub struct Timer {
    start: Instant,
    duration: Duration
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Self { start: Instant::now(), duration }
    }
}

impl Future for Timer {
    type Output = Duration;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if self.start.elapsed() < self.duration {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        } else {
            return Poll::Ready(self.start.elapsed());
        }
    }
}
