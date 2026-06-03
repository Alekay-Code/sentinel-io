use std::future::Future;
use std::cell::RefCell;
use std::rc::Rc;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::task::Waker;

pub struct JoinHandle<T> {
    pub(crate) output: Rc<RefCell<Option<T>>>,
    waker: Rc<RefCell<Option<Waker>>>
}

impl<T: 'static> JoinHandle<T> {
    pub(crate) fn from_future<F>(future: F) -> (JoinHandle<T>, *mut dyn Future<Output = ()>) where F: Future<Output = T> + 'static {
        let handle = JoinHandle { output: Rc::new(RefCell::new(None)), waker: Rc::new(RefCell::new(None)) };
        let state = handle.output.clone();
        let waker = handle.waker.clone();

        let fut = async move {
            let out = future.await;
            state.replace(Some(out));
            if let Some(w) = waker.borrow_mut().take() {
                w.wake();
            }
        };

        let fut = Box::new(fut);
        let fut = Box::into_raw(fut);

        return (handle, fut);
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this   = unsafe { self.get_unchecked_mut() } ;
        let output = this.output.take();

        if output.is_none() {
            this.waker.replace(Some(cx.waker().clone()));
            return std::task::Poll::Pending;
        } else {
            return std::task::Poll::Ready(output.unwrap());
        }
    }
}
