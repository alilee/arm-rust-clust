// SPDX-License-Identifier: Unlicense

#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(libkernel::util::testing::test_runner)]
#![feature(format_args_nl)] // for debug macros

#[allow(unused_imports)]
#[macro_use]
extern crate libkernel;

use test_macros::kernel_test;

#[no_mangle]
fn kernel_init() -> ! {
    test_main();
    unreachable!()
}

#[kernel_test]
fn handler_init() {
    use core::arch::asm;
    use libkernel::handler;

    major!("initialising handler");
    handler::init().expect("handler::init");
    debug!("init'd");

    unsafe { asm!("svc #0") } // no-op

    major!("returned");
}
