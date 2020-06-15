// SPDX-License-Identifier: Unlicense

mod handler;

pub mod mair;

pub use handler::*;

use crate::archs::aarch64::pager;
use crate::pager::{Addr, PhysAddr, Translate, VirtAddr};
use crate::Result;

#[link_section = ".startup"]
#[no_mangle]
#[naked]
/// Entry point for OS
///
/// Positioned at magic address by linker.ld.
///
/// Gets a stack, switches on vm for high memory and calls kernel_init
/// with the first core, and parks the rest.
///
/// NOTE: must not use stack before SP set.
pub extern "C" fn _reset(pdtb: *const u8) -> ! {
    use cortex_a::{asm, regs::*};

    const CORE_0: u64 = 0;
    const AFF0_CORE_MASK: u64 = 0xFF;

    if CORE_0 != MPIDR_EL1.get() & AFF0_CORE_MASK {
        loop {
            asm::wfe();
        }
    }

    extern "C" {
        static STACK_TOP: u64; // defined in linker.ld
    }
    unsafe {
        SP.set(&STACK_TOP as *const u64 as u64);
    }

    enable_vm(pdtb)
}

/// FIXME: Fetch addresses from dtb and calculate
/// TODO: CPACR to enable FP in EL1
/// FIXME: initialise memory
/// TODO: register assignments and translate should be from const (depends on consts in traits)
fn enable_vm(_pdtb: *const u8) -> ! {
    use crate::archs::aarch64;
    use aarch64::pager::TABLE_ENTRIES;
    use cortex_a::{barrier, regs::SCTLR_EL1::*, regs::TCR_EL1::*, regs::*};

    // TODO: device_tree::set(pdtb);

    mair::init();

    #[repr(align(4096))]
    struct TempMap([u64; TABLE_ENTRIES]);

    static mut TEMP_MAP1: TempMap = TempMap([0; TABLE_ENTRIES]);
    static mut TEMP_MAP0: TempMap = TempMap([0; TABLE_ENTRIES]);

    unsafe {
        TEMP_MAP1.0[0] = aarch64::pager::BOOT_RAM_DESCRIPTOR;
        TEMP_MAP1.0[8] = aarch64::pager::BOOT_RAM_DESCRIPTOR;
        TEMP_MAP1.0[10] = aarch64::pager::BOOT_DEVICE_DESCRIPTOR;
        TTBR1_EL1.set(PhysAddr::from_ptr(&TEMP_MAP1).get() as u64);

        TEMP_MAP0.0[0] = aarch64::pager::BOOT_DEVICE_DESCRIPTOR;
        TEMP_MAP0.0[1] = aarch64::pager::BOOT_RAM_DESCRIPTOR;
        TTBR0_EL1.set(PhysAddr::from_ptr(&TEMP_MAP0).get() as u64);
    }

    TCR_EL1.modify(
        AS::Bits_16    // 16 bit ASID
            + IPS::Bits_36  // 36 bits/64GB of physical address space
            + TG1::KiB_4
            + SH1::Outer
            + ORGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T1SZ.val(64 - 39) // L1 1GB pages
            + TG0::KiB_4
            + SH0::Outer
            + ORGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T0SZ.val(64 - 39), // L1 1GB pages
    );

    SCTLR_EL1.modify(M::SET);

    let kernel_offset = unsafe {
        barrier::isb(barrier::SY);

        let kernel_offset = pager::kernel_offset();

        let offset = kernel_offset.offset() as u64;
        asm!("add sp, sp, {}", in(reg) offset);

        kernel_offset
    };

    extern "Rust" {
        fn kernel_init() -> !;
    }

    let phys_addr = unsafe { PhysAddr::from_ptr(kernel_init as *const ()) };
    let kernel_init: fn() -> ! = kernel_offset.translate_phys(phys_addr).unwrap().into();

    kernel_init()
}

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
            + T1SZ.val(64 - super::UPPER_VA_BITS as u64) // 64-t1sz=43 bits of address space in high range
            + TG0::KiB_4
            + SH0::Outer
            + ORGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T0SZ.val(64 - super::LOWER_VA_BITS as u64), // 64-t0sz=48 bits of address space in low range
        );

        // TODO: nTWE nTWI
        SCTLR_EL1.modify(I::SET + C::SET + M::SET);

        barrier::isb(barrier::SY);
    }

    debug!("through!!!");

    Ok(())
}

pub fn set_vbar() -> Result<()> {
    use cortex_a::regs::*;

    extern "C" {
        static vector_table_el1: u8;
    }
    let p_vector_table = unsafe { VirtAddr::from_linker_symbol(&vector_table_el1).get() as u64 };

    VBAR_EL1.set(p_vector_table);

    Ok(())
}
