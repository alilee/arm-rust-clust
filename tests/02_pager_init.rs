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
use libkernel::util::testing::exit_success;

#[no_mangle]
fn kernel_init() {
    test_main();
}

#[kernel_test]
fn paging_init() {
    use libkernel::pager;

    fn next() -> ! {
       exit_success()
    }

    pager::init(next)
}
