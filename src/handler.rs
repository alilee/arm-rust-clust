// SPDX-License-Identifier: Unlicense

//! Register exception handlers and service exceptions.

use crate::Result;

/// Initialise the exception handling module.
pub fn init() -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_call_test_arch() {
        init().expect("init");
    }
}
