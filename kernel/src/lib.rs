//! This is the kernel crate.
//!
//! Responsible for booting the OS, intialising the sub-systems.
//! Also implements the kernel internal API which integrates the submodules.
//! User level API (accessed by supervisor interrupts) is in a submodule.

#![no_std]
#![feature(naked_functions)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(ptr_offset_from)]
#![feature(never_type)]
#![warn(missing_docs)]

mod archs;

#[cfg(test)]
use archs::test as arch;

#[cfg(all(not(test), target_arch = "aarch64"))]
use archs::aarch64 as arch;

#[cfg(all(not(test), target_arch = "arm"))]
use archs::arm as arch;

// Causes this to be exported for linking.
pub use arch::_reset;

mod device;
mod handler;
mod pager;
mod thread;
mod util;

mod user;

#[macro_use]
mod debug;
use debug::uart_logger;

use log::info;

use thread::ThreadID;

/// Kernel API for spawning a new thread
///
/// Integrates the sub-modules.
fn spawn(f: fn() -> ()) -> Result<ThreadID, u64> {
    use thread::Thread;

    let tcb: &mut Thread = Thread::spawn(f)?;
    let stack: [u64; 10] = [0; 10]; // pager::alloc(thread_id, 10)?;
    tcb.set_stack(&stack);
    tcb.ready();
    Ok(tcb.thread_id())
}

/// Kernel function which terminates current thread
///
/// This would be called by a kernel thread to terminate itself.
fn _terminate() -> ! {
    use thread::Thread;
    //
    let t = Thread::current();
    t.terminate();
    // pager::free(thread_id); // what happens to the stack?
    t.unused();

    loop {}
}

/// Boot operating system from first core
///
/// TODO: what happens if any of this code panics?
/// TODO: switch to dedicated EL1 stack for this core
/// TODO: enable other cores
pub fn boot2() -> ! {
    uart_logger::init().unwrap();
    info!("starting");

    // enable virtual memory and map image to kernel virtual range and jump to boot3
    pager::init(boot3)
}

/// Kernel in upper VA
pub fn boot3() -> ! {
    info!("boot3");

    // enable multi-processing and kernel thread
    thread::init();

    // take exceptions
    handler::init();

    // establish io
    device::init();

    let ta = spawn(workload_a).unwrap();
    thread::ready(ta);
    let tb = spawn(workload_b).unwrap();
    thread::ready(tb);

    // clean up boot thread
    thread::terminate()
}

fn panic() -> ! {
    loop {}
}

#[doc(hidden)]
pub fn workload_a() -> () {
    info!("starting workload A");
    loop {
        let mut i = 1000000000u64;
        while i > 0 {
            i = i - 1;
            if i % 42500000 == 0 {
                info!("A")
            }
        }
    }
}

#[doc(hidden)]
pub fn workload_b() -> () {
    info!("starting workload B");
    loop {
        let mut i = 1000000000u64;
        while i > 0 {
            i = i - 1;
            if i % 62500000 == 0 {
                info!("B")
            }
        }
    }
}
