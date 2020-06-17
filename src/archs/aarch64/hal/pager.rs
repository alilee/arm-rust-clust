// SPDX-License-Identifier: Unlicense

//! Implementation of paging on aarch64.

use crate::archs::aarch64;
use crate::Result;

pub fn enable_paging(ttb1: u64, ttb0: u64, asid: u16) -> Result<()> {
    use cortex_a::{
        barrier,
        regs::{SCTLR_EL1::*, TCR_EL1::*, *},
    };

    debug!("enable_paging: {:x}, {:x}, {}", ttb0, ttb1, asid);

    // nothing in low memory except debug device, so no debugging
    unsafe {
        TTBR0_EL1.write(TTBR0_EL1::ASID.val(asid as u64) + TTBR0_EL1::BADDR.val(ttb0 >> 1));
        TTBR1_EL1.write(TTBR1_EL1::ASID.val(asid as u64) + TTBR1_EL1::BADDR.val(ttb1 >> 1));

        TCR_EL1.modify(
            AS::Bits_16    // 16 bit ASID
                + IPS::Bits_36  // 36 bits/64GB of physical address space
                + TG1::KiB_4
                + SH1::Outer
                + ORGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
                + IRGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
                + T1SZ.val(64 - aarch64::UPPER_VA_BITS as u64) // 64-t1sz=43 bits of address space in high range
                + TG0::KiB_4
                + SH0::Outer
                + ORGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
                + IRGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
                + T0SZ.val(64 - aarch64::LOWER_VA_BITS as u64), // 64-t0sz=48 bits of address space in low range
        );

        // TODO: nTWE nTWI
        SCTLR_EL1.modify(I::SET + C::SET + M::SET);

        barrier::isb(barrier::SY);

        asm!("tlbi vmalle1"); // invalidate entire TLB
    }

    debug!("through!!!");

    Ok(())
}

/// Initialise the MAIR register..
///
/// Called during reset, so no debug.
///
/// Note: See pager::mair for offsets.
#[inline(always)]
pub fn init_mair() {
    use cortex_a::regs::{RegisterReadWrite, MAIR_EL1, MAIR_EL1::*};

    MAIR_EL1.write(
        Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck
            + Attr1_Normal_Outer::WriteThrough_NonTransient_ReadWriteAlloc
            + Attr1_Normal_Inner::WriteThrough_NonTransient_ReadWriteAlloc,
    );
}
