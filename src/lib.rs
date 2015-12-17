#![feature(lang_items)]
// #![feature(core_intrinsics)]
// #![feature(core_str_ext)]
// #![feature(core_slice_ext)]
#![feature(type_macros)]
#![feature(const_fn)]
#![no_std]
#![no_main]

#[macro_use]
mod log;

mod uart;

pub mod vm;


#[no_mangle]
pub extern fn rust_main() {

    info!("starting");

    unsafe {
        vm::init();
    }
    
    info!("done, looping.");
    loop {}
    
}


#[cfg(not(test))] 
#[lang = "eh_personality"] extern fn eh_personality() {}

#[cfg(not(test))] 
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop{} }
