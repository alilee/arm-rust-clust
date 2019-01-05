//! This is the kernel crate.
//!
//! Responsible for booting the OS and establishing virtual memory and scheduler.

#![no_std]
#![feature(naked_functions)]
#![feature(uniform_paths)]

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
// mod thread;
// mod pager;
// mod handler;

mod debug;
use debug::uart_logger;

use log::{info};

/// Some documentation.
pub fn boot2() -> ! {

    uart_logger::init().unwrap();
    info!("starting");

    // // take exceptions
    // handler::init();
    // // swap virtual memory
    // pager::init();
    // // enable multi-processing
    // thread::init();
    // // establish io
    device::init();
    //
    // // start the first process
    // thread::spawn(init);
    //
    // // clean up boot process
    // arch::drop_to_userspace();
    // thread::exit();

    arch::loop_forever();
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
