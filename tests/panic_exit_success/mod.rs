// SPDX-License-Identifier: Unlicense

//! Change behaviour of QEMU exit code on panic

/// Overwrites libkernel's `panic::_panic_exit()` with success version.
#[allow(unreachable_code)]
#[no_mangle]
fn _panic_exit() -> ! {
    #[cfg(target_arch = "aarch64")]
    {
        use qemu_exit::QEMUExit;
        qemu_exit::aarch64::AArch64::new().exit_success();
    }

    #[cfg(target_arch = "x86_64")]
    qemu_exit::x86::exit_success();

    unreachable!()
}
