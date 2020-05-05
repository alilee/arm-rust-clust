// SPDX-License-Identifier: Unlicense

#![no_std]
#![feature(naked_functions)]  // for _reset
#![feature(panic_info_message)]

pub mod archs;
pub mod handler;

#[allow(unused_imports)]
use crate::archs::arch::_reset;

use log::info;

/// The default runner for unit tests. Needed to support integration tests.
pub fn test_runner(tests: &[&dyn Fn()]) {
    info!("Running {} tests", tests.len());
    info!("-------------------------------------------------------------------\n");
    for f in tests {
        f();
        info!(".")
    }
}

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
