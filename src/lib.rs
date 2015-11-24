#![feature(no_std, lang_items)]
#![feature(core)]
#![feature(core_intrinsics)]
#![no_std]
#![no_builtins]

// extern crate rlibc;

mod vm;

extern {
    static page_table: *const u32;
    static stack: *const u32;
    static text: *const u32;
    static frame_table: *const u32;
}

#[no_mangle]
pub extern fn rust_main() {
    let x = ["Hello", " ", "World", "!"];
    vm::init();
    vm::id_map(stack, 1);
    vm::id_map(text, 1);
    vm::id_map(page_table, 6);
    vm::id_map(frame_table, 1);
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! {loop{}}
