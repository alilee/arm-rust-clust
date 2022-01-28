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
    use libkernel::{handler, pager};

    fn next() -> ! {
        test_main();
        unreachable!()
    }

    handler::init().expect("handler::init");
    pager::init(next)
}

#[kernel_test]
fn paging_init() {
    assert!(true)
}

use libkernel::debug::Level;

#[no_mangle]
pub fn _override_log_settings() -> (Level, &'static [(&'static str, Level)]) {
    const LOG_LEVEL_SETTINGS: &[(&str, Level)] = &[];
    (Level::Trace, LOG_LEVEL_SETTINGS)
}
