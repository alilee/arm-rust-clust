//! This is the kernel crate.
//!
//! Responsible for booting the OS and establishing virtual memory and scheduler.

#![warn(missing_docs)]
#![feature(const_fn)]
#![feature(asm)]

#![no_std]

extern crate rlibc;

mod archs;

#[cfg(target_arch = "aarch64")]
use archs::aarch64 as arch;

#[cfg(target_arch = "arm")]
use archs::arm as arch;

// Causes this to be exported.
pub use arch::handler::handler;

mod dev;
mod thread;
mod vmm;

mod dbg;
use dbg::uart_logger;

#[macro_use]
extern crate log;

/// Some documentation.
#[no_mangle]
pub extern "C" fn boot2() -> ! {

    uart_logger::init().unwrap();

    info!("starting");

    // assume we're starting our own cluster
    arch::handler::init();

    // 1. set up scheduling
    //    boot2 is this thread, now EL0, must be cleaned up
    thread::init();
    thread::spawn(init);
    thread::discard_boot();
}

fn init() -> () {

    // test: should be able to get back to EL1 at this point
    // arch::svc(10);

    // 2. start vmm
    //   map live kernel into fixed va
    //   vbar table
    //   exception handlers
    vmm::init();

    // start device discovery
    //   blk: backing store
    //   con:
    //   start login task on consoles
    dev::init();

    thread::spawn(workload);
}

#[doc(hidden)]
pub fn workload() -> () {
    loop {
        info!("working...");
        let mut i = 1000000000u64;
        while i > 0 {
            i = i - 1;
        }
    }
}
