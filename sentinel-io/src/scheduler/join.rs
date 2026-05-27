use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub(crate) struct JoinState<T> {
    pub(crate) output: Option<T>,
    pub(crate) waker: Option<Waker>,
}

pub struct JoinHandle<T> {
    pub(crate) state: Arc<Mutex<JoinState<T>>>,
}

impl<T> JoinHandle<T> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(JoinState {
                output: None,
                waker: None,
            })),
        }
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut state = self.state.lock().unwrap();
        match state.output.take() {
            Some(v) => Poll::Ready(v),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
