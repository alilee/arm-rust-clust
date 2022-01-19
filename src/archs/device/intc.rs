// SPDX-License-Identifier: Unlicense

//! Interface for architecture-specific interrupt controller.

use crate::Result;

use dtb::StructItems;

pub(super) fn init(_dtb_root: StructItems) -> Result<()> {
    major!("init");
    error!("no intc!");
    Ok(())
}
