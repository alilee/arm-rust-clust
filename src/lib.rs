#![feature(lang_items)]
// #![feature(core_intrinsics)]
// #![feature(core_str_ext)]
// #![feature(core_slice_ext)]
#![feature(type_macros)]
#![feature(const_fn)]
#![no_std]
#![no_main]

extern crate aeabi;

mod uart;
pub mod vm;

#[macro_use]
mod log;

// extern {
//     static page_table: *const u32;
//     static stack: *const u32;
//     static text: *const u32;
//     static frame_table: *const u32;
// }

// #[no_mangle]
// pub extern fn rust_main2() {
//     vm::id_map(stack as u32, 1);
//     vm::id_map(text as u32, 1);
//     vm::id_map(page_table as u32, 6);
//     vm::id_map(frame_table as u32, 1);
// }
//

use uart::UART0;
use core::fmt::Write;

#[no_mangle]
pub extern fn rust_main() {

    info!("starting");

    unsafe {
        vm::init();
    }

    info!("done");
    
    unsafe {
        
        let res = aeabi::__aeabi_uidivmod(27, 5);
        write!(uart::UART0, "({},{})", res.0, res.1);

    }
    
    loop {}
    
}


#[cfg(not(test))] 
#[lang = "eh_personality"] extern fn eh_personality() {}

#[cfg(not(test))] 
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop{} }
