// SPDX-License-Identifier: Unlicense

//! Wrapper for locking and releasing a mutex through a local variable.

use spin::{Mutex, MutexGuard};

/// Wraps a generic object in a Mutex.
pub struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    /// Create a Mutex wrapping an object.
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    /// Hold the lock on the Mutex while local variable is live.
    ///
    /// NOTE: Cannot log here as debug Uart is a Locked object.
    pub fn lock(&self) -> MutexGuard<A> {
        self.inner.lock()
    }
}
