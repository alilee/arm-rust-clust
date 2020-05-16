//! This is the kernel crate.
//!
//! Responsible for booting the OS, intialising the sub-systems.
//! Also implements the kernel internal API which integrates the submodules.
//! User level API (accessed by supervisor interrupts) is in a submodule.

#![no_std]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(ptr_offset_from)]
#![feature(never_type)]
#![feature(alloc_error_handler)]
#![warn(missing_docs)]

extern crate alloc;

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
// mod handler;
// mod heap;
mod pager;
// mod range;
// mod thread;
mod util;

mod user;

#[macro_use]
mod debug;
use debug::uart_logger;

use log::info;

/// Boot operating system from first core. Called from arch::_reset.
///
/// TODO: switch to dedicated EL1 stack for this core
/// TODO: enable other cores
pub fn boot2() -> ! {
    uart_logger::init().unwrap();
    info!("starting");

    // enable virtual memory, map image to kernel virtual range and jump to boot3
    // pager::init(boot3)
}

/// Executes at kernel VA
// pub fn boot3() -> ! {
//     info!("boot3");
//
//     // take exceptions
//     handler::init();
//
//     // support alloc and collections
//     heap::init().expect("failed initialising heap");
//
//     // enable multi-processing and kernel thread
//     thread::init();
//
//     // establish io
//     device::init();
//
//     let ta = thread::spawn(workload_a).unwrap();
//     thread::ready(ta);
//     let tb = thread::spawn(workload_b).unwrap();
//     thread::ready(tb);
//
//     thread::show_state();
//
//     // clean up boot thread and yield to ready workload
//     thread::terminate()
// }

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
