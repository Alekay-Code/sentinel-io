use std::cell::RefCell;
use std::mem;
use std::ptr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, RawWaker, RawWakerVTable, Waker};
use std::collections::VecDeque;

use super::task::task::Task;
use super::task::join::JoinHandle;
use super::scheduler::{self, Scheduler};

use super::super::context::CONTEXT;

thread_local! {
    static TASKS: RefCell<VecDeque<Pin<Arc<Mutex<Task>>>>> = RefCell::new(VecDeque::new());
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
    return TASKS.with_borrow_mut(|queue| queue.pop_front());
}

fn push_task(task: Pin<Arc<Mutex<Task>>>) {
    TASKS.with_borrow_mut(|queue| queue.push_back(task));
}

fn desk_size() -> usize {
    TASKS.with_borrow_mut(|queue| queue.len())
}

pub fn block_on<F>(main: F)
where
    F: Future<Output = ()> + 'static,
{
    let wrapper = async move {
        main.await;
    };

    let main_task = Arc::new(Mutex::new(Task::new(wrapper)));
    let main_ptr  = Arc::as_ptr(&main_task);
    push_task(Pin::new(main_task));

    'outer: loop {
        while let Some(task) = pop_task() {
            // Get pointer of the task
            let task_ptr = Arc::as_ptr(&Pin::into_inner(task.clone()));

            let waker_arc = Pin::into_inner(task.clone());
            let raw = Arc::into_raw(waker_arc) as *const ();
            let waker = unsafe { Waker::from_raw(RawWaker::new(raw, &VTABLE)) };
            let mut cx = Context::from_waker(&waker);

            let mut guard = task.lock().unwrap();
            let pinned = unsafe { Pin::new_unchecked(&mut *guard) };

            let state = pinned.poll(&mut cx);

            if ptr::eq(main_ptr, task_ptr) && state.is_ready() {
                break 'outer
            }
        }

        if desk_size() == 0 {
            break;
        }
    }
}

pub struct Runtime {
    scheduler: Scheduler
}

impl Runtime {
    pub fn new(scheduler: Scheduler) -> Runtime {

    }

    pub fn block_on<F>(&mut self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        // Scheduler Plan
        match &mut self.scheduler {
            Scheduler::SingleThread(s) => s.block_on(fut),
        }
    }
}
