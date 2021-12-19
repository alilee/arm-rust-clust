// SPDX-License-Identifier: Unlicense

//! Code executed on system reset.

use core::arch::asm;

use crate::pager::{Addr, PhysAddr, Translate};

use tock_registers::interfaces::{ReadWriteable, Writeable};

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
pub unsafe extern "C" fn reset(pdtb: *const u8) -> ! {
    asm!(
        "   mrs x1, mpidr_el1",
        "   and x1, x1, 0xFF", // aff0
        "   cbz x1, 3f",       // core 0
        "2: wfe",
        "   b 2b", // TODO: init remaining cores
        "3: adrp x1, STACK_TOP",
        "   mov sp, x1",
        "   b enable_boot_vm",
        options(noreturn)
    )
}

#[allow(dead_code)]
#[no_mangle]
/// FIXME: Fetch addresses from dtb and calculate
/// TODO: CPACR to enable FP in EL1
/// FIXME: initialise memory
/// TODO: register assignments and translate should be from const (depends on consts in traits)
extern "C" fn enable_boot_vm(_pdtb: *const u8) -> ! {
    use crate::archs::aarch64;
    use aarch64::pager::TABLE_ENTRIES;
    use cortex_a::{asm::barrier, registers::SCTLR_EL1::*, registers::TCR_EL1::*, registers::*};

    // TODO: device_tree::set(pdtb);

    super::init_mair();

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
        AS::ASID16Bits
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

        let kernel_offset = aarch64::pager::kernel_offset();

        let offset = kernel_offset.offset() as u64;
        asm!("add sp, sp, {}", in(reg) offset);

        kernel_offset
    };

    extern "Rust" {
        fn kernel_init() -> !; // address relative to _reset, not high memory
    }

    let phys_addr = unsafe { PhysAddr::from_ptr(kernel_init as *const ()) };
    let kernel_init: fn() -> ! = kernel_offset.translate_phys(phys_addr).unwrap().into();

    kernel_init()
}
