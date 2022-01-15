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

#[no_mangle]
pub extern "C" fn collect_tests() -> () {
    test_main()
}

#[inline(never)]
#[kernel_test]
fn heap_test() {
    info!("allocating");
    let a = Box::new([1u64; 2000]);
    let pa: *const u64 = &a[0];
    info!("allocated: {:?}", pa);

    let mut total = 0u64;
    for i in a.iter() {
        total += i;
    }
    info!("finished iterating");

    assert_eq!(total, 2000);
}

use libkernel::debug::Level;

#[no_mangle]
fn _override_log_levels() -> (Level, &'static [(&'static str, Level)]) {
    const LOG_LEVEL_SETTINGS: &[(&str, Level)] = &[
        ("aarch64::pager", Level::Major),
        ("pager::layout", Level::Major),
        ("pager::frames", Level::Major),
    ];
    (Level::Trace, LOG_LEVEL_SETTINGS)
}
