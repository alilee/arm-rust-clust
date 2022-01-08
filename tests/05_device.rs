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
fn kernel_init() {
    use libkernel::{handler, pager};

    handler::init().expect("handler::init");
    pager::init().expect("pager::init");
    pager::alloc::init().expect("pager::alloc::init");

    test_main();
}

#[kernel_test]
fn device_init() {
    use libkernel::device;

    major!("initialising device");
    device::init().expect("device::init");
    debug!("returned");
}
