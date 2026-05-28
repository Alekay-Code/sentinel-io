use std::pin::Pin;
use std::task::Waker;

use super::super::task::task::Task;

pub struct Worker;

impl Worker {
    pub fn new() -> Worker {
        Worker
    }

    pub fn execute(&self, task: &mut Task, waker: &Waker) -> bool {
        let pinned = unsafe { Pin::new_unchecked(task) };
        let mut cx = std::task::Context::from_waker(waker);
        pinned.poll(&mut cx).is_ready()
    }
}
