// SPDX-License-Identifier: Unlicense

#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(libkernel::util::testing::test_runner)]
#![feature(format_args_nl)] // for debug macros
#![allow(unused_imports)]

#[macro_use]
extern crate libkernel;

#[macro_use]
extern crate claim;

extern crate alloc;

use libkernel::debug::_breakpoint;
use libkernel::pager::{Addr, Page, VirtAddr, PAGESIZE_BYTES};
use test_macros::kernel_test;

#[no_mangle]
pub extern "C" fn collect_tests() -> () {
    info!("hello collect_tests");
    test_main()
}

#[kernel_test]
fn test_heap() {
    info!("test_heap");
    use alloc::boxed::Box;

    panic!("hello");

    _breakpoint();

    let x = Box::new(1);
    info!("new: {:?}", &*x as *const i32);

    assert_eq!(*x, 1);
    info!("returning");
}
