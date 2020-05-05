// SPDX-License-Identifier: Unlicense

#![no_std]
#![no_main]

extern crate libkernel; // yes, this is needed

use libkernel::*;

#[no_mangle]
fn kernel_init() -> ! {
    kernel_main()
}

fn kernel_main() -> ! {
    handler::init();
    unreachable!()
}
