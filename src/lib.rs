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
    // UART0.write_str(file!());
    // UART0.write_str(line!());
    // UART0.write_str(module_path!());

    info!("starting");

    //
    // // unsafe {
    // //     vm::init();
    // // }
    //
    // info!("done");
    
    loop {}
    
}


#[cfg(not(test))] 
#[lang = "eh_personality"] extern fn eh_personality() {}

#[cfg(not(test))] 
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop{} }
