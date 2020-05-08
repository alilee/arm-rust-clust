// SPDX-License-Identifier: Unlicense

#![no_std]
#![no_main]
#![feature(format_args_nl)] // for debug macros

extern crate libkernel; // yes, this is needed

use libkernel::*;

#[no_mangle]
fn kernel_init() -> ! {
    info!("starting");

    // enable virtual memory, map image to kernel virtual range and jump to boot3
    // pager::init(kernel_main)
    kernel_main()
}

fn kernel_main() -> ! {
    info!("kernel_main");

    // handler::init().expect("handler::init");
    // heap::init().expect("heap::init");
    // thread::init().expect("thread::init");
    // device::init().expect("device::init");
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

    loop {}
}
