// SPDX-License-Identifier: Unlicense

//! Kernel as library, to facilitate integration testing.

#![no_std]
#![feature(naked_functions)] // for _reset
#![feature(panic_info_message)]
#![feature(format_args_nl)]  // for debug logging macros
#![feature(const_fn)] // casting pointer to ints in PhysAddr::from_linker_symbol
#![feature(const_raw_ptr_to_usize_cast)] // casting pointer to ints in PhysAddr::from_linker_symbol
#![warn(missing_docs)]

#[macro_use]
pub mod debug;

pub mod archs;
pub mod device;
pub mod handler;
pub mod pager;
pub mod util;

pub use util::result::{Result, Error};

#[allow(unused_imports)]
use crate::archs::arch::_reset;

#[doc(hidden)]
#[cfg(not(test))]
pub mod lang_items {

    /// Log panic information and abnormal-exit emulator (or hang)
    #[allow(unreachable_code)]
    #[panic_handler]
    fn panic(info: &core::panic::PanicInfo) -> ! {
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
