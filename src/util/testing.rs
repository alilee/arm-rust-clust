// SPDX-License-Identifier: Unlicense

//! Support for integration testing, shared between tests.

use crate::debug;

/// The runner for integration tests.
///
/// NOTE: This is not used for unit tests.
pub fn test_runner(tests: &[&test_types::UnitTest]) {
    debug::uart_logger::init().expect("debug::uart_logger");
    use log::info;

    info!("running {} tests", tests.len());
    for test in tests {
        info!("testing {}", test.name);
        (test.test_func)();
    }
    info!("test result: ok.");

    qemu_exit::aarch64::exit_success();
}
