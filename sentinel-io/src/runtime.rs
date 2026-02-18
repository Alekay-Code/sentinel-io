use std::collections::HashMap;
use std::sync::mpsc::RecvTimeoutError;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};
use std::pin::Pin;
use std::task::Poll;
use std::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::mem;
use std::time::Duration;

const CHANNEL_SIZE: usize = 10;
type Id = usize;

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(data: *const ()) -> RawWaker {
    unsafe {
        let task = Arc::from_raw(data as *const Task);
        let c = task.clone();
        mem::forget(task);
        return RawWaker::new(Arc::into_raw(c) as *const (), &VTABLE)
    }
}

unsafe fn wake(data: *const ()) {
    unsafe {
        let task = Arc::from_raw(data as *const Task);
        let _ = task.chan_send.send(task.id);
    }
}

unsafe fn wake_by_ref(data: *const ()) {
    unsafe {
        let task = Arc::from_raw(data as *const Task);
        let _ = task.chan_send.send(task.id);
        mem::forget(task);
    }
}

unsafe fn drop(data: *const ()) {
    unsafe {
        let task = Arc::from_raw(data as *const Task);
        mem::drop(task);
    }
}

struct Task {
    id: Id,
    future: Pin<Box<dyn Future<Output = ()>>>,
    sleep: bool,
    chan_send: mpsc::SyncSender<Id>
}

impl Task {
    fn wake_task(&self) {
        let _ = self.chan_send.send(self.clone_as_arc());
    }

    fn create_waker(self: &Arc<Self>) -> Waker {
        let ptr = Arc::into_raw(self.clone()) as *const ();
        let raw_waker = RawWaker::new(ptr, &VTABLE);
        unsafe { Waker::from_raw(raw_waker) }
    }
}

pub struct Runtime {
    id_counter: Id,
    tasks: Vec<Arc<Mutex<Task>>>,
    chan_recv: mpsc::Receiver<Id>,
    chan_send: mpsc::SyncSender<Id>,
}

impl Runtime {
    pub fn new() -> Self {
        let (chan_send, chan_recv) = mpsc::sync_channel(CHANNEL_SIZE);
        Self { id_counter: 0, tasks: Vec::new(), sleep: HashMap::new(), chan_send, chan_recv }
    }

    pub fn push(&mut self, fut: Pin<Box<dyn Future<Output = ()>>>, sleep: bool) {
        let mut waker = Waker::noop();
        let ctx = Context::from_waker(&waker);

        let task = Arc::new(Mutex::new(Task { id: self.id_counter, future: fut, sleep, chan_send: self.chan_send.clone() }));

        if sleep {
            let raw_waker = RawWaker::new(Arc::into_raw(task.clone()) as *const (), &VTABLE);
            unsafe {
                waker = Waker::from_raw(raw_waker);
            }
        }

        task.lock().as_mut().unwrap().ctx = Context::from_waker(waker);
        self.id_counter += 1;
        self.tasks.push(task);
    }

    pub fn run(&mut self) {
        loop {
            if self.tasks.len() <= 0 && self.sleep.len() <= 0 {
                break;
            }

            let mut i = 0;

            while i < self.tasks.len() {
                let mut task_guard = self.tasks[i].lock().unwrap();
                let task = &mut *task_guard;

                match task.future.as_mut().poll(&mut task.ctx) {
                    Poll::Ready(_) => {
                        mem::drop(task_guard);
                        self.tasks.remove(i);
                    },

                    Poll::Pending => {
                        if task.sleep {
                            let id = task.id;
                            mem::drop(task_guard);
                            let task = self.tasks.remove(i);
                            self.sleep.insert(id, task);
                        } else {
                            i += 1;
                        }
                    },
                }
            }

            if self.sleep.len() > 0 {
                let res = self.chan_recv.recv_timeout(Duration::from_secs(0));
                match res {
                    Ok(id) => {
                        self.tasks.push(self.sleep.remove(&id).unwrap())
                    },

                    Err(RecvTimeoutError::Timeout) => {},
                    Err(e) => eprintln!("Error: {e}")
                }
            }
        }
    }
}
