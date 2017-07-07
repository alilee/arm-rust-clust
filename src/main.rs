#![feature(lang_items)]

#![no_main]
#![no_std]

extern crate kernel;

#[cfg(not(test))]
pub mod lang_items {
    #[lang = "panic_fmt"]
    #[no_mangle] // FIXME: https://github.com/rust-lang/rust/issues/38281
    pub extern "C" fn panic_fmt() -> ! {
        loop {}
    }
}
