use super::worker::Worker;
use super::super::task::task::Task;
use super::super::task::join::JoinHandle;

pub struct SingleThread {
    worker: Worker
}

impl SingleThread {
    pub fn new() -> SingleThread {
        SingleThread { worker: Worker::new() }
    }

    pub fn block_on<F>(&mut self, fut: F)
    where
        F: Future + 'static,
    {
        let handler = JoinHandle::new();

        let future = async move {
            let result = fut.await;
            let mut state = handler.state.lock().unwrap();
            state.output = Some(result);
        };

        let mut task = Task::new(future);

        while !self.worker.execute(&mut task) {}
    }
}
