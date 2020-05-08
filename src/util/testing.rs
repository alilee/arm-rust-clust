// SPDX-License-Identifier: Unlicense

//! Support for integration testing, shared between tests.

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

    qemu_exit::aarch64::exit_success();
}
