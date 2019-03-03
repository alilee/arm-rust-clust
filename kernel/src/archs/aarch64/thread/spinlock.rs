
use core::sync::atomic::{Ordering, AtomicBool};

#[inline]
fn acquire(spinlock: &mut AtomicBool) {
    while spinlock.swap(true, Ordering::SeqCst) {}
}

#[inline]
fn release(spinlock: &mut AtomicBool) {
    spinlock.store(false, Ordering::SeqCst);
}

// struct Critical<'a> {
//     b: &'a mut AtomicBool,
// }
//
// impl<'a> Critical<'a> {
//     pub fn new(b: &'a mut AtomicBool) -> Critical {
//         acquire(b);
//         Critical { b: b }
//     }
// }
//
// impl<'a> Drop for Critical<'a> {
//     fn drop(&mut self) {
//         release(self.b);
//     }
// }

pub const fn new() -> AtomicBool {
    AtomicBool::new(false)
}

pub fn exclusive<F, T>(b: &mut AtomicBool, closure: F) -> T
    where F: FnOnce() -> T {
    acquire(b);
    let res = closure();
    release(b);
    res
}
