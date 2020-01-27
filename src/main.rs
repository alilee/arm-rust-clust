//! A proxy to build the library with particular link settings.

#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate kernel; // yes, this is needed

#[doc(hidden)]
#[cfg(not(test))]
pub mod lang_items {
    #[panic_handler]
    fn panic(info: &core::panic::PanicInfo) -> ! {
        use log::error;
        match info.location() {
            None => error!(
                "Panic: {}",
                info.message().unwrap_or(&format_args!("unknown"))
            ),
            Some(loc) => error!(
                "Panic: {} (at {}:{})",
                info.message().unwrap_or(&format_args!("unknown")),
                loc.file(),
                loc.line()
            ),
        };

        #[cfg(target_arch = "aarch64")]
        qemu_exit::aarch64::exit_failure();

        #[allow(unreachable_code)]
        loop {}
    }
}

#[test]
pub fn test_one() {
    assert!(false);
}
