//! The thread module manages the set of local TCBs.
//!
//! A TCB is a space for thread state to be stored when not executing, to
//! pass messages for IPC, and a stack.
//!

/// To enable waiting.
pub struct JoinHandle<T>(T);

/// Start a new thread at an entry point.
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    JoinHandle(f())
}

/// Initialise the thread system on boot.
pub fn init() -> () {}

/// Safely discard the boot thread.
pub fn discard_boot() -> ! {

    unreachable!()
}
