// SPDX-License-Identifier: Unlicense

#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(libkernel::util::testing::test_runner)]
#![feature(format_args_nl)] // for debug macros

#[macro_use]
extern crate libkernel;

#[no_mangle]
fn kernel_init() {
    test_main();
}

mod tests {
    use test_macros::kernel_test;

    #[kernel_test]
    fn a_test() {
        assert!(true)
    }

    #[kernel_test]
    fn b_test() {
        info!("hello");
        assert!(true)
    }
}
