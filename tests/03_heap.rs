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

use libkernel::pager::{Addr, Page, VirtAddr, PAGESIZE_BYTES};
use test_macros::kernel_test;

#[no_mangle]
fn kernel_init() {
    test_main();
}

#[kernel_test]
fn test_heap() {
    info!("test_heap");
    use alloc::boxed::Box;

    let backing = Page::new();
    debug!("backing");

    unsafe {
        let mut heap = libkernel::pager::alloc::ALLOCATOR.lock();
        debug!("lock");
        heap.init(VirtAddr::from(&backing).get(), PAGESIZE_BYTES);
        debug!("init");
    }

    let x = Box::new(1);
    debug!("new");

    assert_eq!(*x, 1);
    debug!("returning");
}
