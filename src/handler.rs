// SPDX-License-Identifier: Unlicense

//! Register exception handlers and service exceptions.

use crate::archs::{ArchTrait, arch::Arch};

/// Initialise the exception handling module.
pub fn init() {
    Arch::init_handler();
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::unit_test;

    #[unit_test]
    fn it_works() {
        init();
        assert!(true)
    }

    #[unit_test]
    fn another_works() {
        log::info!("another_works");
        assert!(true);
    }
}