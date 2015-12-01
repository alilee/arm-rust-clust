#![feature(no_std, lang_items)]
#![feature(core)]
#![feature(core_intrinsics)]
#![feature(core_str_ext)]
#![no_std]

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! { loop{} }

mod aeabi;
mod uart;
// mod vm;

// extern {
//     static page_table: *const u32;
//     static stack: *const u32;
//     static text: *const u32;
//     static frame_table: *const u32;
// }

// #[no_mangle]
// pub extern fn rust_main2() {
//     let x = ["Hello", " ", "World", "!"];
//     vm::init();
//     vm::id_map(stack as u32, 1);
//     vm::id_map(text as u32, 1);
//     vm::id_map(page_table as u32, 6);
//     vm::id_map(frame_table as u32, 1);
// }
//

#[no_mangle]
pub extern fn rust_main() {

    uart::puts("hello world\n");
    
}

