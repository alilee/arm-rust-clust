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

#[macro_use]
extern crate claim;

use libkernel::util::testing::exit_success;
use test_macros::kernel_test;

#[no_mangle]
fn kernel_init() {
    test_main();
}

fn next() -> ! {
    use libkernel::archs::{arch::Arch, PagerTrait};
    use libkernel::pager::*;

    info!("!!!");

    unsafe {
        assert_gt!(PhysAddr::from_fn(next).get(), Arch::kernel_base().get());
    }

    exit_success()
}

#[kernel_test]
fn paging_init() {
    use libkernel::pager;

    pager::init(next)
}
