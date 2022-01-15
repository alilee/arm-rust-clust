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

use test_macros::kernel_test;

#[no_mangle]
pub extern "C" fn collect_tests() -> () {
    test_main()
}

#[macro_use]
extern crate claim;

extern crate alloc;

#[kernel_test]
fn test_heap() {
    info!("test_heap");
    use alloc::boxed::Box;

    let x = Box::new(1);
    info!("new: {:?}", &*x as *const i32);

    assert_eq!(*x, 1);
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
