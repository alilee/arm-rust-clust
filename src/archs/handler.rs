// SPDX-License-Identifier: Unlicense

//! Interface for architecture-specific exception handling.

use crate::Result;

/// Each architecture must supply the following entry points for paging..
pub trait HandlerTrait {
    /// Initialise exception handling.
    fn handler_init() -> Result<()>;

    /// Loop forever
    fn wait_forever() -> ! {
        unimplemented!()
    }
}
