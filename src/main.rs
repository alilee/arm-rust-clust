// SPDX-License-Identifier: Unlicense

#![no_std]
#![no_main]
#![feature(format_args_nl)] // for debug macros

extern crate libkernel; // yes, this is needed

use libkernel::*;

/// Kernel entry point, called from architecture-specific reset.
#[no_mangle]
fn kernel_init() -> ! {
    major!("starting");

    handler::init().expect("handler::init");
    pager::init(kernel_main)
}

/// Kernel in high memory, initialise rest of kernel.
fn kernel_main() -> ! {
    major!("kernel_main");

    // heap::init().expect("heap::init");
    device::init().expect("device::init");
    // thread::init().expect("thread::init");
    //
    // let ta = thread::spawn(workload_a).unwrap();
    // thread::ready(ta);
    // let tb = thread::spawn(workload_b).unwrap();
    // thread::ready(tb);
    //
    // thread::show_state();
    //
    // // clean up boot thread and yield to ready workload
    // thread::terminate()

    major!("looping");
    loop {}
}
