// SPDX-License-Identifier: Unlicense

//! Implementation of MAIR.

/// Initialise the MAIR register.
///
/// Called during reset, so no debug.
///
/// Note: See pager::mair for offsets.
#[inline(always)]
pub fn init() {
    use cortex_a::regs::{RegisterReadWrite, MAIR_EL1, MAIR_EL1::*};

    MAIR_EL1.write(
        Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck
            + Attr1_Normal_Outer::WriteThrough_NonTransient_ReadWriteAlloc
            + Attr1_Normal_Inner::WriteThrough_NonTransient_ReadWriteAlloc,
    );
}
