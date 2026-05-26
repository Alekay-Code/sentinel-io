use std::cell::RefCell;
use std::mem;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

use super::join::JoinHandle;
use super::task::Task;

thread_local! {
    static DESK: RefCell<Vec<Pin<Arc<Mutex<Task>>>>> = RefCell::new(Vec::new());
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(data: *const ()) -> RawWaker {
    unsafe { Arc::increment_strong_count(data as *const Mutex<Task>) };
    RawWaker::new(data, &VTABLE)
}

unsafe fn wake(data: *const ()) {
    unsafe {
        let arc = Arc::from_raw(data as *const Mutex<Task>);
        let task = Pin::new_unchecked(arc);
        push_task(task);
    }
}

unsafe fn wake_by_ref(data: *const ()) {
    unsafe {
        let arc = Arc::from_raw(data as *const Mutex<Task>);
        let task = Pin::new_unchecked(arc.clone());
        mem::forget(arc);
        push_task(task);
    }
}

unsafe fn drop(data: *const ()) {
    unsafe { Arc::from_raw(data as *const Mutex<Task>) };
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
{
    let handler = JoinHandle::new();
    let state = Arc::clone(&handler.state);

    let wrapper = async move {
        let output = future.await;
        let mut s = state.lock().unwrap();
        s.output = Some(output);
        if let Some(waker) = s.waker.take() {
            waker.wake();
        }
    };

    let task = Arc::pin(Mutex::new(Task::new(wrapper)));
    push_task(task);

    handler
}

fn pop_task() -> Option<Pin<Arc<Mutex<Task>>>> {
    return DESK.with_borrow_mut(|queue| queue.pop());
}

fn push_task(task: Pin<Arc<Mutex<Task>>>) {
    DESK.with_borrow_mut(|queue| queue.push(task));
}

fn desk_size() -> usize {
    DESK.with_borrow_mut(|queue| queue.len())
}


pub fn block_on<F>(main: F)
where
    F: Future<Output = ()> + 'static,
{
    // TODO: Create a waker for main future to suspend the execution until all task are completed
    let main_task = Arc::new(Mutex::new(Task::new(main)));
    push_task(Pin::new(main_task));

    while desk_size() > 0 {
        while let Some(task) = pop_task() {
            let waker_arc = Pin::into_inner(task.clone());
            let raw = Arc::into_raw(waker_arc) as *const ();
            let waker = unsafe { Waker::from_raw(RawWaker::new(raw, &VTABLE)) };
            let mut cx = Context::from_waker(&waker);

            let mut guard = task.lock().unwrap();
            let pinned = unsafe { Pin::new_unchecked(&mut *guard) };

            let _ = pinned.poll(&mut cx);
        }
    }
}
