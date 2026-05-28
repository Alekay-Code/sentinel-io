use std::cell::RefCell;
use std::sync::Arc;

pub(crate) mod worker;
pub mod single_thread;

use single_thread::SingleThread;
use super::task::task::Task;

pub enum Scheduler {
    SingleThread(SingleThread),
}

impl Scheduler {
    pub fn push_task(&mut self, task: Arc<RefCell<Task>>) {
        match self {
            Scheduler::SingleThread(s) => s.push_task(task),
        }
    }

    pub fn pop_task(&mut self) -> Option<Arc<RefCell<Task>>> {
        match self {
            Scheduler::SingleThread(s) => s.pop_task(),
        }
    }
}
