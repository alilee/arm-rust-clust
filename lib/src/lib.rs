#![feature(const_fn)]
#![feature(asm)]

#![no_std]

extern crate rlibc;

mod archs;

#[cfg(target_arch = "aarch64")]
use archs::aarch64 as arch;

#[cfg(target_arch = "arm")]
use archs::arm as arch;

mod dev;
mod exc;
// mod vm;

mod dbg;
use dbg::uart_logger;

#[macro_use]
extern crate log;

#[no_mangle]
pub extern "C" fn rust_main() {

    uart_logger::init().unwrap();

    info!("starting");

    // assume we're starting our own cluster

    // map live kernel into fixed va
    //   vbar table
    //   exception handlers
    //
    // start device discovery
    //   blk: backing store
    //   con:
    //   start login task on consoles
    //
    // vm::init();
    //

    arch::drop_to_userspace();

    workload();

    loop_forever();
    uart_logger::shutdown().unwrap();

}

fn loop_forever() {
    info!("done, looping..");
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

pub fn workload() {
    loop {
        info!("working...");
        let mut i = 1000000000u64;
        while i > 0 {
            i = i - 1;
        }
    }
}
