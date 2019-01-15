

use log::info;

use core::sync::atomic::{Ordering, AtomicBool};


#[inline]
fn acquire(spinlock: &mut AtomicBool) {
    while spinlock.swap(true, Ordering::SeqCst) {}
}

#[inline]
fn release(spinlock: &mut AtomicBool) {
    spinlock.store(false, Ordering::SeqCst);
}


struct Spinlock<'a> {
    b: &'a mut AtomicBool,
}

impl<'a> Spinlock<'a> {
    pub fn new(b: &'a mut AtomicBool) -> Spinlock {
        acquire(b);
        Spinlock { b: b }
    }
}

impl<'a> Drop for Spinlock<'a> {
    fn drop(&mut self) {
        release(self.b);
    }
}


pub fn exclusive<F, T>(b: &mut AtomicBool, closure: F) -> T
    where F: FnOnce() -> T {
    acquire(b);
    let res = closure();
    release(b);
    res
}
