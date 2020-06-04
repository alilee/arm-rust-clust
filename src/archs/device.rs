// SPDX-License-Identifier: Unlicense

//! Interface for architecture-specific device functions..

use crate::pager::{PhysAddrRange};
use crate::Result;

/// Each architecture must supply the following entry points for paging..
pub trait DeviceTrait {
    /// Initialise device management.
    fn device_init() -> Result<()> {
        info!("init");
        Ok(())
    }

    /// Return the physical address range of the UART for debug log.
    fn debug_uart() -> Result<PhysAddrRange>;
}

