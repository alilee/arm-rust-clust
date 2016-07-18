#![feature(lang_items)]
#![feature(const_fn)]
#![feature(asm)]

#![no_std]

extern crate rlibc;

mod archs;
use archs::armv8 as arch;

mod dev;
mod exc;
// mod vm;

mod dbg;
use dbg::uart_logger;

#[macro_use]
extern crate log;

#[no_mangle]
pub extern fn rust_main() {

    uart_logger::init().unwrap();
    
    info!("starting");
    arch::hello();
    
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
    
    // exc::drop_to_EL0();
    
    info!("EL0");
    
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

fn workload() {
    loop {
        info!("working...");
        let mut i = 1000000000u64;
        while i > 0 {
            i = i - 1;
        }
    }
}

#[cfg(not(test))]
#[lang = "eh_personality"] extern fn eh_personality() {}

#[cfg(not(test))]
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! { loop{} }

#[cfg(not(test))]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}