use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

use super::worker::Worker;
use super::super::task::task::Task;

pub struct SingleThread {
    pub(crate) queue: VecDeque<Arc<RefCell<Task>>>,
}

impl SingleThread {
    pub fn new() -> SingleThread {
        SingleThread {
            queue: VecDeque::new(),
        }
    }

    pub fn push_task(&mut self, task: Arc<RefCell<Task>>) {
        self.queue.push_back(task);
    }

    pub fn pop_task(&mut self) -> Option<Arc<RefCell<Task>>> {
        self.queue.pop_front()
    }
}
