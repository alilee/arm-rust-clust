// SPDX-License-Identifier: Unlicense

//! Register exception handlers and service exceptions.

use crate::Result;

/// Initialise the exception handling module.
pub fn init() -> Result<()> {
    use crate::archs::{arch::Arch, HandlerTrait};
    Arch::handler_init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_call_test_arch() {
        init().expect("init");
    }
}
