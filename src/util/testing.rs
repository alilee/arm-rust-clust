// SPDX-License-Identifier: Unlicense

//! Support for integration testing, shared between tests.

/// Quit the kernel and emulator signalling success.
#[allow(unreachable_code)]
pub fn exit_success() -> ! {
    #[cfg(target_arch = "aarch64")]
    qemu_exit::aarch64::exit_success();

    #[cfg(target_arch = "x86_64")]
    qemu_exit::x86::exit_success();

    loop {}
}

/// The runner for integration tests.
///
/// NOTE: This is not used for unit tests.
pub fn test_runner(tests: &[&test_types::UnitTest]) {
    info!("running {} tests", tests.len());
    for test in tests {
        info!("testing {}", test.name);
        (test.test_func)();
    }
    info!("test result: ok.");

    exit_success()
}
