use std::task::{RawWaker, Waker, RawWakerVTable};
use std::future::Future;
use super::runtime;

pub static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop_waker);

unsafe fn clone(ptr: *const ()) -> RawWaker {
    let task = unsafe { *(ptr as *const *mut dyn Future<Output = ()>) };
    let boxed = Box::into_raw(Box::new(task));
    RawWaker::new(boxed as *const (), &VTABLE)
}

unsafe fn wake(ptr: *const ()) {
    let boxed = ptr as *mut *mut dyn Future<Output = ()>;
    let task  = unsafe { *boxed };
    unsafe { drop(Box::from_raw(boxed)) };
    runtime::add_task(task);
}

unsafe fn wake_by_ref(ptr: *const ()) {
    let task = unsafe { *(ptr as *const *mut dyn Future<Output = ()>) };
    runtime::add_task(task);
}

unsafe fn drop_waker(ptr: *const ()) {
    unsafe { drop(Box::from_raw(ptr as *mut *mut dyn Future<Output = ()>)) };
}

pub fn make_waker(task: *mut dyn Future<Output = ()>) -> Waker {
    let boxed = Box::into_raw(Box::new(task));
    unsafe { Waker::from_raw(RawWaker::new(boxed as *const (), &VTABLE)) }
}
