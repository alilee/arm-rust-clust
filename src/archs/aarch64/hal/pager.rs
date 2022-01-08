// SPDX-License-Identifier: Unlicense

//! Implementation of paging on aarch64.

use core::arch::asm;

use crate::archs::aarch64;
use crate::Result;

use super::handler::EsrEL1;

use cortex_a::registers::*;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

/// Initialise the MAIR register..
///
/// Called during reset, so no debug.
///
/// Note: See pager::mair for offsets.
#[inline(always)]
pub fn init_mair() {
    use cortex_a::registers::MAIR_EL1::*;

    MAIR_EL1.write(
        Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck
            + Attr1_Normal_Outer::WriteThrough_NonTransient_ReadWriteAlloc
            + Attr1_Normal_Inner::WriteThrough_NonTransient_ReadWriteAlloc,
    );
}

///
pub fn enable_paging(ttb1: u64, ttb0: u64, asid: u16) -> Result<()> {
    use cortex_a::{
        asm::barrier,
        registers::{SCTLR_EL1::*, TCR_EL1::*, *},
    };

    debug!("enable_paging: {:x}, {:x}, {}", ttb0, ttb1, asid);

    // nothing in low memory except debug device, so no debugging
    unsafe {
        TTBR0_EL1.write(TTBR0_EL1::ASID.val(asid as u64) + TTBR0_EL1::BADDR.val(ttb0 >> 1));
        TTBR1_EL1.write(TTBR1_EL1::ASID.val(asid as u64) + TTBR1_EL1::BADDR.val(ttb1 >> 1));

        TCR_EL1.modify(
            AS::ASID16Bits
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

#[inline(never)]
/// Set the stack pointer
pub fn move_stack(stack_pointer: usize) -> () {
    unsafe {
        asm!("mov sp, {}",
             in(reg) stack_pointer)
    }
}

/// EL1 has triggered a data abort
///
/// Page table entry for read or write access is invalid - either paged out
/// or was never mapped.
pub fn handle_data_abort_el1(esr: LocalRegisterCopy<u64, ESR_EL1::Register>) -> Result<()> {
    use crate::pager::{Addr, VirtAddr};
    use ESR_EL1::ISS_DATA_FAULT_STATUS_CODE_REASON::Value;
    info!("handle_data_abort_el1");

    let dfsc_reason: Value = esr
        .read_as_enum(ESR_EL1::ISS_DATA_FAULT_STATUS_CODE_REASON)
        .expect("unknown ESR_EL1::ISS_DATA_FAULT_STATUS_CODE_REASON");
    match dfsc_reason {
        Value::Translation => {
            let fault_addr = VirtAddr::at(FAR_EL1.get() as usize);
            crate::pager::kernel_translation_fault(
                fault_addr,
                Some(esr.read(ESR_EL1::ISS_DATA_FAULT_STATUS_CODE_LEVEL)),
            )
        }
        Value::AccessFlag => unimplemented!(),
        _ => unimplemented!(),
    }
}
