// SPDX-License-Identifier: Unlicense

//! Device trait for aarch64.

use super::Arch;

use crate::archs::DeviceTrait;
use crate::pager::{Addr, AddrRange, HandlerReturnAction, PhysAddr, PhysAddrRange};
use crate::Result;

impl DeviceTrait for Arch {
    fn add_handler(_interrupt: u8, _handler: fn() -> HandlerReturnAction) -> Result<()> {
        todo!()
    }

    fn debug_uart() -> Result<PhysAddrRange> {
        Ok(PhysAddrRange::new(PhysAddr::at(0x900_0000), 0x1000))
    }
}
