#![feature(lang_items)]
#![feature(const_fn)]
#![feature(asm)]

#![no_std]

extern crate rlibc;

mod uart;
mod uart_logger;

#[macro_use]
extern crate log;

// pub mod vm;

#[no_mangle]
pub extern "C" fn rust_main() {

    uart_logger::init().unwrap();

    info!("starting");
    // vm::init();
    //
    // error!("test error");
    // warn!("test warn");
    // info!("test info");
    // debug!("test debug");
    // trace!("test trace");
    //

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

// #[cfg(not(test))]
// #[lang = "eh_personality"]
// extern "C" fn eh_personality() {}

#[cfg(not(test))]
pub mod lang_items {
    #[lang = "panic_fmt"]
    #[no_mangle]
    extern "C" fn panic_fmt() -> ! {
        loop {}
    }
}

// #[cfg(not(test))]
// #[allow(non_snake_case)]
// #[no_mangle]
// pub extern "C" fn _Unwind_Resume() -> ! {
//     loop {}
// }
