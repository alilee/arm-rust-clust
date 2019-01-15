//! This is the kernel crate.
//!
//! Responsible for booting the OS and establishing virtual memory and scheduler.

#![no_std]
#![feature(naked_functions)]
#![feature(uniform_paths)]
#![feature(global_asm)]
#![feature(asm)]

#![warn(missing_docs)]

mod archs;

#[cfg(target_arch = "aarch64")]
use archs::aarch64 as arch;

#[cfg(target_arch = "arm")]
use archs::arm as arch;

// Causes this to be exported for linking.
// pub use arch::handler::handler;
pub use arch::_reset;

mod device;
mod thread;
// mod pager;
mod handler;

mod debug;
use debug::uart_logger;

use log::{info};

/// Some documentation.
///
/// TODO: what happens if any of this code panics?
pub fn boot2() -> ! {

    uart_logger::init().unwrap();
    info!("starting");

    // take exceptions
    handler::init();
    handler::supervisor();

    // // swap virtual memory
    // pager::init();

    // enable multi-processing
    thread::init();
    // establish io
    device::init();

    // start the first process
    spawn(workload).unwrap();

    // clean up boot process
    // arch::drop_to_userspace();
    terminate();
    // thread is cleaned up and core should shift to other thread... until that terminates.
}


/// Kernel API for spawning a new thread
///
/// Integrates the sub-modules.
fn spawn(f: fn() -> ()) -> Result<u64, u64> {
    let tcb = thread::ControlBlock::spawn(f)?;
    let stack: [u64; 10] = [0; 10]; // pager::alloc(thread_id, 10)?;
    tcb.set_stack(stack)?;
    tcb.ready()?;
    Ok(tcb.thread_id())
}


fn terminate() -> ! {
    //
    let thread_id: thread::ThreadID = thread::current();
    thread::terminate(thread_id);
    // pager::free(thread_id); // what happens to the stack?
    thread::yield();
}


// fn init() -> () {
//
//     // test: should be able to get back to EL1 at this point
//     // arch::svc(10);
//
//     thread::spawn(workload);
//
//     arch::loop_forever()
// }

#[doc(hidden)]
pub fn workload() -> () {
    loop {
        // info!("working...");
        let mut i = 1000000000u64;
        while i > 0 {
            i = i - 1;
        }
    }
}
