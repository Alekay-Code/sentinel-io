use std::sync::mpsc::RecvTimeoutError;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};
use std::pin::Pin;
use std::task::Poll;
use std::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::mem;
use std::time::{Duration, Instant};
use std::thread;

const CHANNEL_SIZE: usize = 10;

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(data: *const ()) -> RawWaker {
    unsafe {
        let task = Arc::from_raw(data as *const Mutex<Task>);
        let c = task.clone();
        mem::forget(task);
        return RawWaker::new(Arc::into_raw(c) as *const (), &VTABLE)
    }
}

unsafe fn wake(data: *const ()) {
    unsafe {
        let task = Arc::from_raw(data as *const Mutex<Task>);
        let _ = task.lock().unwrap().chan_send.send(task.clone());
    }
}

unsafe fn wake_by_ref(data: *const ()) {
    unsafe {
        let task = Arc::from_raw(data as *const Mutex<Task>);
        let task_clone = task.clone();
        let guard = task.lock().unwrap();
        let _ = guard.chan_send.send(task_clone);
        mem::drop(guard);
        mem::forget(task);
    }
}

unsafe fn drop(data: *const ()) {
    unsafe {
        let task = Arc::from_raw(data as *const Mutex<Task>);
        mem::drop(task);
    }
}

fn simulate_wake_up(waker: Waker) {
    waker.wake();
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    sleep: bool,
    chan_send: mpsc::SyncSender<Arc<Mutex<Self>>>,
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

    pub fn push(&mut self, fut: Pin<Box<dyn Future<Output = ()>>>, sleep: bool) {
        let task = Arc::new(Mutex::new(Task { future: fut, sleep, chan_send: self.chan_send.clone() }));
        self.tasks.push(task);
    }

    pub fn run(&mut self) {
        let time = Instant::now();
        loop {
            if self.tasks.len() <= 0 && self.tasks_sleeping <= 0 {
                break;
            }

            let mut i = 0;

            let res = self.chan_recv.recv_timeout(Duration::from_millis(0));
            match res {
                // Insert task into the list
                Ok(task) => {
                    self.tasks.push(task);
                    self.tasks_sleeping -= 1;
                },

                // Ignore timeout
                Err(RecvTimeoutError::Timeout) => {},
                Err(e) => eprintln!("ERROR: {e}")
            }

            while i < self.tasks.len() {
                let task = &mut self.tasks[i].clone();
                let task_clone = task.clone();
                let mut task = task.lock().unwrap();


                let task_ptr = Arc::into_raw(task_clone) as *const();

                unsafe {
                    let waker = Waker::from_raw(RawWaker::new(task_ptr, &VTABLE));
                    let waker_for_thread = waker.clone();
                    let mut ctx = Context::from_waker(&waker);

                    match task.future.as_mut().poll(&mut ctx) {
                        Poll::Ready(_) => {
                            self.tasks.remove(i);
                        },

                        Poll::Pending => {
                            if task.sleep {
                                _ = self.tasks.remove(i);
                                self.tasks_sleeping += 1;

                                thread::spawn(move || {
                                    println!("OS WORKING: {}", time.elapsed().as_secs());
                                    thread::sleep(Duration::from_secs(10));
                                    println!("OS WAKE: {}", time.elapsed().as_secs());
                                    simulate_wake_up(waker_for_thread);
                                });

                            } else {
                                i += 1;
                            }
                        },
                    }
                }
            }
        }
    }
}
