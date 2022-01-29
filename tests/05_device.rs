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
pub extern "C" fn collect_tests() -> () {
    test_main()
}

#[kernel_test]
fn device_init() {
    use libkernel::device;

    major!("initialising device");
    device::init().expect("device::init");
    debug!("returned");
}

use libkernel::debug::Level;

#[no_mangle]
fn _override_log_levels() -> (Level, &'static [(&'static str, Level)]) {
    const LOG_LEVEL_SETTINGS: &[(&str, Level)] = &[
        ("aarch64::pager", Level::Info),
        ("pager", Level::Info),
        ("pager::layout", Level::Major),
        ("pager::frames", Level::Info),
        ("pager::bump", Level::Major),
        ("pager::alloc", Level::Major),
    ];
    (Level::Trace, LOG_LEVEL_SETTINGS)
}
