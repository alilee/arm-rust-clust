#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(libkernel::test_runner)]

extern crate libkernel;

// use log::trace;

#[no_mangle]
fn kernel_init() -> ! {
    test_main();
    qemu_exit::aarch64::exit_success()
}

#[test_case]
fn a_test() {
    assert!(true)
}

#[test_case]
fn b_test() {
    assert!(false)
}