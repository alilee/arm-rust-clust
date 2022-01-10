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

use libkernel::util::testing::exit_success;
use test_macros::kernel_test;

#[no_mangle]
fn kernel_init() {
    test_main();
}

#[kernel_test]
fn paging_init() {
    use libkernel::{handler, pager};

    _breakpoint();

    fn next() -> ! {
        info!("!!!");
        exit_success()
    }

    handler::init().expect("handler::init");
    pager::init(next)
}

use libkernel::debug::{Level, _breakpoint};

#[no_mangle]
pub fn _is_enabled(level: Level, module_path: &str) -> bool {
    const LOG_LEVEL_SETTINGS: &[(&str, Level)] = &[("aarch64::pager", Level::Info)];

    let setting = LOG_LEVEL_SETTINGS
        .into_iter()
        .fold(Level::Trace, |base, (pat, level)| {
            if module_path.ends_with(pat) {
                *level
            } else {
                base
            }
        });
    true || level >= setting
}
