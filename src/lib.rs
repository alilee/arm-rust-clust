#![feature(no_std, lang_items)]
#![feature(core_intrinsics)]
#![feature(core_str_ext)]
#![feature(core_slice_ext)]
#![no_std]

#[cfg(not(test))] // missing in libcore, supplied by libstd
#[lang = "eh_personality"] extern fn eh_personality() {}

#[cfg(not(test))] // missing in libcore, supplied by libstd
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! { loop{} }

extern crate aeabi;

mod uart;
pub mod vm;

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

#[no_mangle]
pub extern fn rust_main() {

    vm::init();

    uart::puts("hello world\n");
    
}

