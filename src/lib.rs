#![feature(no_std, lang_items)]
#![feature(core)]
#![feature(core_intrinsics)]
#![no_std]

// extern crate rlibc;

mod vm;
mod aeabi;

extern {
    static page_table: *const u32;
    static stack: *const u32;
    static text: *const u32;
    static frame_table: *const u32;
}

#[no_mangle]
pub extern fn rust_main2() {
    let x = ["Hello", " ", "World", "!"];
    vm::init();
    vm::id_map(stack as u32, 1);
    vm::id_map(text as u32, 1);
    vm::id_map(page_table as u32, 6);
    vm::id_map(frame_table as u32, 1);
}

#[no_mangle]
pub extern fn rust_main() {
    
    let gpio = 0x3F200000 as *mut u32;
    let LED_GPFBIT = (1 << 21);
    let LED_GPIO_BIT = (1 << 15);
    
    unsafe {
        *gpio.offset(4) |= LED_GPFBIT;

        loop {
    
            for i in 0..500000 {
                let x = 0;
            } 

            *gpio.offset(11) = LED_GPIO_BIT;

            for i in 0..500000 {
                let x = 0;
            } 

            *gpio.offset(8) = LED_GPIO_BIT;
        
        }
    }
}


#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! {loop{}}
