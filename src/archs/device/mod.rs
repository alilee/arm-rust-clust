// SPDX-License-Identifier: Unlicense

//! Interface for architecture-specific device functions..

mod intc;

use crate::pager::{HandlerReturnAction, PhysAddrRange};
use crate::Result;

use dtb::StructItems;

/// Each architecture must supply the following entry points for paging..
pub trait DeviceTrait {
    /// Initialise architecture-specific devices - interrupt controller
    fn device_init(dtb_root: StructItems) -> Result<()> {
        major!("init");
        intc::init(dtb_root)
    }

    /// Add an interrupt handler
    fn add_handler(_interrupt: u8, _handler: fn() -> HandlerReturnAction) -> crate::Result<()>;

    /// Return the physical address range of the UART for debug log.
    fn debug_uart() -> Result<PhysAddrRange>;
}
