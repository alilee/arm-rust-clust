// SPDX-License-Identifier: Unlicense

//! Panic handling.

#[cfg(not(test))]
use crate::archs::{ArchTrait, arch::Arch};

/// The point of exit for the "standard" (non-testing) `libkernel`.
///
/// This code will be used by the release kernel binary and the `integration tests`. It is linked
/// weakly, so that the integration tests can overload it to exit `QEMU` instead of spinning
/// forever.
#[cfg(not(test))]
#[linkage = "weak"]
#[no_mangle]
#[allow(unreachable_code)]
fn _panic_exit() -> ! {
    #[cfg(target_arch = "aarch64")]
    qemu_exit::aarch64::exit_failure();

    #[cfg(target_arch = "x86_64")]
    qemu_exit::x86::exit_failure();

    Arch::wait_forever()
}

/// Log panic information and abnormal-exit emulator (or hang)
#[cfg(not(test))]
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

    _panic_exit();
}
