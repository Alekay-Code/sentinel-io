use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use super::join::JoinHandle;

pub struct Task {
    pub future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        Self {
            future: Box::pin(future),
        }
    }
}

impl Future for Task {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let future = &mut self.get_mut().future;
        return future.as_mut().poll(cx);
    }
}
