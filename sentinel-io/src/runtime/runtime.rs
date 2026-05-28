use std::cell::{Cell, RefCell};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::task::{RawWaker, RawWakerVTable, Waker};

use super::task::task::Task;
use super::task::join::JoinHandle;
use super::scheduler::Scheduler;
use super::scheduler::single_thread::SingleThread;
use super::scheduler::worker::Worker;
use super::super::context::Context;

pub(crate) struct RuntimeInner {
    scheduler: Scheduler,
}

#[derive(Clone)]
pub struct Runtime {
    inner: Rc<RefCell<RuntimeInner>>,
}

// --- Waker VTABLE ---
// Waker data: *const RefCell<Task> obtained from Arc::into_raw.
// wake() reconstructs the Arc and re-queues the task via Context.

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    waker_clone,
    waker_wake,
    waker_wake_by_ref,
    waker_drop,
);

unsafe fn waker_clone(data: *const ()) -> RawWaker {
    unsafe { Arc::increment_strong_count(data as *const RefCell<Task>) };
    RawWaker::new(data, &VTABLE)
}

unsafe fn waker_wake(data: *const ()) {
    // Reconstructs the Arc that the waker owned and re-queues the task
    let arc = unsafe { Arc::from_raw(data as *const RefCell<Task>) };
    Context::with_runtime(|rt| rt.push_task(Arc::clone(&arc)));
    // arc drops here — the clone in the queue now owns the reference
}

unsafe fn waker_wake_by_ref(data: *const ()) {
    // Increment before wake so the caller's reference is not consumed
    unsafe { Arc::increment_strong_count(data as *const RefCell<Task>) };
    unsafe { waker_wake(data) };
}

unsafe fn waker_drop(data: *const ()) {
    // Reconstruct and immediately drop to decrement the refcount
    let _ = unsafe { Arc::from_raw(data as *const RefCell<Task>) };
}

// --- Runtime ---

impl Runtime {
    pub fn new() -> Runtime {
        let inner = Rc::new(RefCell::new(RuntimeInner {
            scheduler: Scheduler::SingleThread(SingleThread::new()),
        }));
        let rt = Runtime { inner };
        Context::init_runtime(rt.clone());
        rt
    }

    pub(crate) fn push_task(&self, task: Arc<RefCell<Task>>) {
        self.inner.borrow_mut().scheduler.push_task(task);
    }

    pub fn block_on<F>(&self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        // Flag set when the root future completes — break as soon as it does,
        // without waiting for tasks spawned but not awaited.
        let root_done = Rc::new(Cell::new(false));
        let root_done_flag = Rc::clone(&root_done);

        let root = async move {
            fut.await;
            root_done_flag.set(true);
        };

        self.push_task(Arc::new(RefCell::new(Task::new(root))));

        loop {
            // Pop — borrow released before polling so spawn() can push freely
            let task_arc = {
                let mut inner = self.inner.borrow_mut();
                inner.scheduler.pop_task()
            };

            let Some(task_arc) = task_arc else { break };

            // Build waker — Arc::clone increments refcount for the waker's lifetime
            let raw = Arc::into_raw(Arc::clone(&task_arc)) as *const ();
            let waker = unsafe { Waker::from_raw(RawWaker::new(raw, &VTABLE)) };

            // Poll — borrows RefCell<Task>, not RefCell<RuntimeInner>, so no conflict
            // if the future calls spawn() which needs to borrow RuntimeInner
            {
                let mut task_guard = task_arc.borrow_mut();
                Worker::new().execute(&mut *task_guard, &waker);
            }

            if root_done.get() {
                break;
            }
        }
    }
}

// --- spawn ---

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
{
    let handle = JoinHandle::new();
    let state = Arc::clone(&handle.state);

    let wrapper = async move {
        let output = future.await;
        let mut s = state.lock().unwrap();
        s.output = Some(output);
        if let Some(waker) = s.waker.take() {
            waker.wake();
        }
    };

    let task = Arc::new(RefCell::new(Task::new(wrapper)));
    Context::with_runtime(|rt| rt.push_task(Arc::clone(&task)));

    handle
}

// Free block_on for the #[sentinel::main] macro
pub fn block_on<F>(fut: F)
where
    F: Future<Output = ()> + 'static,
{
    Runtime::new().block_on(fut);
}
