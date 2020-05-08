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

    #[test]
    fn it_works() {
        init();
        assert!(true)
    }

    #[test]
    fn another_works() {
        info!("another_works");
        assert!(true);
    }
}