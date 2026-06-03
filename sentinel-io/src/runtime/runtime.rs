use std::future::Future;
use std::collections::VecDeque;
use std::cell::RefCell;
use std::pin::Pin;
use std::task::Context;
use std::ptr;

use super::vtable;
use super::join::JoinHandle;

thread_local! {
    static TASK: RefCell<VecDeque<*mut dyn Future<Output= ()>>> = RefCell::new(VecDeque::new());
}

pub(crate) fn add_task(task: *mut dyn Future<Output= ()>) {
    TASK.with_borrow_mut(|tasks| { tasks.push_back(task) } );
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output> where F: Future + 'static {
    let (handle, fut) = JoinHandle::from_future(future);
    TASK.with(|task| { task.borrow_mut().push_back(fut); } );
    return handle;
}

pub fn block_on<F>(future: F) where F: Future<Output = ()> + 'static {
    let main = async {
        future.await;
    };

    let main           = Box::new(main);
    let main_ptr       = Box::into_raw(main);

    TASK.with_borrow_mut(|task| { task.push_back(main_ptr); } );

    loop {
        let task = TASK.with_borrow_mut(|tasks| { tasks.pop_front() });

        if let Some(fut) = task {
            let waker = vtable::make_waker(fut);
            let mut cx = Context::from_waker(&waker);
            let pinned_fut = unsafe { Pin::new_unchecked(&mut *fut) };

            if pinned_fut.poll(&mut cx).is_ready() {
                // free memory, future is completed
                unsafe { let _ = Box::from_raw(fut); };

                if ptr::eq(fut, main_ptr) {
                    break
                }
            }

        } else {
            // There aren't more tasks to execute
            break
        }
    }
}
