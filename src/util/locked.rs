// SPDX-License-Identifier: Unlicense

//! Wrapper for locking and releasing a mutex through a local variable.

use spin::Mutex;

/// Sync access to a static variable.
pub type Locked<A> = Mutex<A>;
