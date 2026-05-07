use std::task::{Context, RawWaker, RawWakerVTable, Waker};
use std::pin::Pin;
use std::task::Poll;
use std::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::mem;
use std::future::Future;
use std::time::Duration;

const CHANNEL_SIZE: usize = 10;

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(data: *const ()) -> RawWaker {
    unsafe {
        let wd = Arc::from_raw(data as *const WakerData);
        let c = wd.clone();
        mem::forget(wd);
        return RawWaker::new(Arc::into_raw(c) as *const (), &VTABLE)
    }
}

unsafe fn wake(data: *const ()) {
    unsafe {
        let wd = Arc::from_raw(data as *const WakerData);
        let _ = wd.chan_send.send(wd.task.clone());
    }
}

unsafe fn wake_by_ref(data: *const ()) {
    println!("wake_by_ref");
    unsafe {
        let wd = Arc::from_raw(data as *const WakerData);
        let _ = wd.chan_send.send(wd.task.clone());
        mem::forget(wd);
    }
}

unsafe fn drop(data: *const ()) {
    unsafe {
        let wd = Arc::from_raw(data as *const WakerData);
        mem::drop(wd);
    }
}

struct WakerData {
    task: Arc<Mutex<Task>>,
    chan_send: mpsc::SyncSender<Arc<Mutex<Task>>>,
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Runtime {
    tasks: Vec<Arc<Mutex<Task>>>,
    tasks_sleeping: usize,
    chan_recv: mpsc::Receiver<Arc<Mutex<Task>>>,
    chan_send: mpsc::SyncSender<Arc<Mutex<Task>>>,
}

impl Runtime {
    pub fn new() -> Self {
        let (chan_send, chan_recv) = mpsc::sync_channel(CHANNEL_SIZE);
        Self { tasks: Vec::new(), tasks_sleeping: 0, chan_send, chan_recv }
    }

    pub fn run(&mut self) {
        loop {
            println!("loop");
            if self.tasks.len() <= 0 && self.tasks_sleeping <= 0 {
                break;
            }

            if self.tasks_sleeping > 0 {
                println!("waiting");
                let res = self.chan_recv.recv_timeout(Duration::from_secs(3));
                match res {
                    // Insert task into the list
                    Ok(task) => {
                        self.tasks.push(task);
                        self.tasks_sleeping -= 1;
                    },

                    // Ignore timeout
                    Err(_e) => {},
                }
            }

            while let Some(task) = self.tasks.pop() {
                println!("Task");
                let wd = Arc::new(WakerData {
                    task: task.clone(),
                    chan_send: self.chan_send.clone(),
                });
                let wd_ptr = Arc::into_raw(wd) as *const ();

                let mut guard = task.lock().unwrap();
                unsafe {
                 let waker = Waker::from_raw(RawWaker::new(wd_ptr, &VTABLE));
                 let mut ctx = Context::from_waker(&waker);

                 match guard.future.as_mut().poll(&mut ctx) {
                     Poll::Ready(_) => {},
                     Poll::Pending => {
                        self.tasks_sleeping += 1;
                    }
                }
                }
            }
        }
    }

    /// Block until the future is completed
    fn block_on<F>(&self, fut: F) -> F::Output where F: Future {
        let waker = Waker::noop();
        let mut ctx = Context::from_waker(waker);
        let mut pin_fut = Box::pin(fut);

        loop {
            match pin_fut.as_mut().poll(&mut ctx) {
                Poll::Pending => continue,
                Poll::Ready(v) => return v
            }
        }
    }

    /// Spawn a new task
    pub fn spawn<F>(&mut self, fut: F) where F: Future<Output = ()> + 'static {
        let f = Box::pin(fut);
        let task = Arc::new(Mutex::new(Task { future: f }));
        self.tasks.push(task);
    }
}
