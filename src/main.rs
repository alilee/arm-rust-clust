//! A proxy to build the library with particular link settings.

#![no_std]

extern crate kernel;

#[doc(hidden)]
#[cfg(not(test))]
pub mod lang_items {
    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> ! {
        loop {}
    }
}

#[test]
pub fn test_one() {
    assert!(false);
}
