// SPDX-License-Identifier: Unlicense

//! Implementation of MAIR.

use crate::Result;

/// Initialise the MAIR register.
///
/// Note: See pager::mair for offsets.
pub fn init() -> Result<()> {
    use cortex_a::regs::{RegisterReadWrite, MAIR_EL1, MAIR_EL1::*};

    MAIR_EL1.write(
        Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck
            + Attr1_Normal_Outer::WriteThrough_NonTransient_ReadWriteAlloc
            + Attr1_Normal_Inner::WriteThrough_NonTransient_ReadWriteAlloc,
    );

    trace!("init -> MAIR_EL1 {:#b}", MAIR_EL1.get());

    Ok(())
}
