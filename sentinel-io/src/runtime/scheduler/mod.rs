pub (crate) mod worker;

pub mod single_thread;
use single_thread::SingleThread;

pub enum Scheduler {
    SingleThread(SingleThread)
}
