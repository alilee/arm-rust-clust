// SPDX-License-Identifier: Unlicense

#![no_std]
#![feature(naked_functions)] // for _reset
#![feature(panic_info_message)]

pub mod archs;
pub mod debug;
pub mod device;
pub mod handler;
pub mod util;

#[allow(unused_imports)]
use crate::archs::arch::_reset;

use log::info;

/// The runner for integration tests.
pub fn test_runner(tests: &[&test_types::UnitTest]) {
    debug::uart_logger::init().expect("debug::uart_logger");

    info!("running {} tests", tests.len());
    for test in tests {
        info!("testing {}", test.name);
        (test.test_func)();
    }
    info!("test result: ok.");

    qemu_exit::aarch64::exit_success();
}

#[doc(hidden)]
#[cfg(not(test))]
pub mod lang_items {

    /// Log panic information and abnormal-exit emulator (or hang)
    #[allow(unreachable_code)]
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

        #[cfg(target_arch = "x86_64")]
        qemu_exit::x86::exit_failure();

        loop {}
    }
}
