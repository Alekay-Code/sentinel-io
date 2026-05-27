
// Global context
use super::Runtime;
use super::runtime::scheduler::{self, Scheduler};
use crate::runtime::scheduler::single_thread::SingleThread;

use std::cell::Cell;

pub struct Context {
    runtime: Cell<Runtime>
    scheduler: Cell<Scheduler>
}


thread_local! {
    pub static CONTEXT: Context = Context {
        runtime: Cell::new(Runtime::new()),
        scheduler: Cell::new(Scheduler::SingleThread(SingleThread))
    }
}
