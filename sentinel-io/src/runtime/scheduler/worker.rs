use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Waker};

use super::super::task::join::JoinHandle;
use super::super::task::task::Task;

pub struct Worker {
    // TODO: Use allocated memory instead of vector
    tasks: VecDeque<Task>,
    current: Option<Task>
}

impl Worker {
    pub fn new() -> Worker {
        Worker { tasks: VecDeque::new(), current: None }
    }

    pub fn remaining_tasks(&self) -> usize {
        self.tasks.len()
    }

    pub fn add_tasks(&mut self, new_tasks: &mut VecDeque<Task>) {
        self.tasks.append(new_tasks);
    }

    pub fn execute(&mut self, task: &mut Task) -> bool {
        unsafe {
            let t = Pin::new_unchecked(task);

            // TODO: Use proper context
            let waker = Waker::noop();
            let mut cx = Context::from_waker(waker);
            match t.poll(&mut cx) {
                std::task::Poll::Ready(_) => true,
                _ => false
            }
        }
    }

    // pub fn block_on<F>(&mut self, fut: F) -> JoinHandle<F::Output>
    // where
    //     F: Future<Output = ()> + 'static,
    // {
    //     let handle = JoinHandle::new();
    //
    //     // Execute futures
    //
    //     return handle;
    // }
}
