// SPDX-License-Identifier: Unlicense

#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(libkernel::util::testing::test_runner)]
#![feature(format_args_nl)]
#![feature(layout_for_ptr)] // for debug macros

#[allow(unused_imports)]
#[macro_use]
extern crate libkernel;

extern crate alloc;
use alloc::boxed::Box;

use test_macros::kernel_test;

mod panic_exit_success;

#[no_mangle]
fn kernel_init() {
    use libkernel::{handler, pager};

    fn next() -> ! {
        test_main();
        unreachable!()
    }

    handler::init().expect("handler::init");
    pager::init(next);
}

#[inline(never)]
#[kernel_test]
fn heap_test() {
    use core::alloc::{GlobalAlloc, Layout};
    use core::ptr;

    use libkernel::pager;

    unsafe {
        let t: *const [u64; 1000] = core::ptr::null();
        let result = pager::alloc::ALLOCATOR.alloc(Layout::for_value_raw(t));
        assert_eq!(result, ptr::null_mut());
    }

    info!("allocating");
    let a = Box::new([1u64; 1000]);
    let pa: *const u64 = &a[0];
    info!("allocated: {:?}", pa);
    let mut total = 0u64;
    for i in a.iter() {
        total += i;
    }
    info!("finished iterating");
    assert_eq!(total, 1000);
}
