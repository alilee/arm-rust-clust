// SPDX-License-Identifier: Unlicense

#![no_std]
#![no_main]

extern crate libkernel; // yes, this is needed

use libkernel::*;

use log::info;

#[no_mangle]
fn kernel_init() -> ! {
    debug::uart_logger::init().expect("debug::uart_logger");
    info!("starting");

    kernel_main()
}

fn kernel_main() -> ! {
    handler::init();
    unreachable!()
}
