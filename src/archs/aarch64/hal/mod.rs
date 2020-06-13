mod handler;

pub mod mair;

pub use handler::*;

use crate::pager::{Addr, FixedOffset, PhysAddr, Translate, VirtAddr};
use crate::Result;

#[link_section = ".startup"]
#[no_mangle]
#[naked]
/// Entry point for OS
///
/// Positioned at magic address by linker.ld.
///
/// Gets a stack and calls boot2 for the first core, and parks the rest in a WFE loop.
///
/// NOTE: must not use stack before SP set.
///
/// FIXME: Fetch addresses from dtb and calculate
/// TODO: CPACR to enable FP in EL1
pub extern "C" fn _reset(_pdtb: *const u8) -> ! {
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

    // device_tree::set(pdtb);
    enable_vm()
}

fn enable_vm() -> ! {
    use crate::archs::aarch64;
    use aarch64::pager::TABLE_ENTRIES;
    use cortex_a::{barrier, regs::SCTLR_EL1::*, regs::TCR_EL1::*, regs::*};

    #[repr(align(4096))]
    struct TempMap([u64; TABLE_ENTRIES]);

    static mut TEMP_MAP1: TempMap = TempMap([0; TABLE_ENTRIES]);
    static mut TEMP_MAP0: TempMap = TempMap([0; TABLE_ENTRIES]);

    unsafe {
        TEMP_MAP1.0[0] = aarch64::pager::make_boot_ram_descriptor();
        TEMP_MAP1.0[8] = aarch64::pager::make_boot_ram_descriptor();
        TEMP_MAP1.0[10] = aarch64::pager::make_boot_device_descriptor();
        TTBR1_EL1.set(PhysAddr::from_ptr(&TEMP_MAP1).get() as u64);

        TEMP_MAP0.0[0] = aarch64::pager::make_boot_device_descriptor();
        TEMP_MAP0.0[1] = aarch64::pager::make_boot_ram_descriptor();
        TTBR0_EL1.set(PhysAddr::from_ptr(&TEMP_MAP0).get() as u64);
    }

    TCR_EL1.modify(
        AS::Bits_16    // 16 bit ASID
            + IPS::Bits_36  // 36 bits/64GB of physical address space
            + TG1::KiB_4
            + SH1::Outer
            + ORGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T1SZ.val(64 - super::UPPER_VA_BITS as u64) // L1 1GB pages
            + TG0::KiB_4
            + SH0::Outer
            + ORGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T0SZ.val(64 - super::UPPER_VA_BITS as u64), // L1 1GB pages
    );

    SCTLR_EL1.modify(M::SET);

    let kernel_offset = unsafe {
        barrier::isb(barrier::SY);

        let kernel_offset = FixedOffset::new(
            PhysAddr::from_ptr(_reset as *const ()),
            VirtAddr::at(0xffff_ff82_0008_0000),
        );

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

pub fn enable_paging(ttb1: u64, ttb0: u64, asid: u16, offset: usize) -> Result<()> {
    use cortex_a::{
        barrier,
        regs::{SCTLR_EL1::*, TCR_EL1::*, *},
    };

    TTBR0_EL1.write(TTBR0_EL1::ASID.val(asid as u64) + TTBR0_EL1::BADDR.val(ttb0 >> 1));
    TTBR1_EL1.write(TTBR1_EL1::ASID.val(asid as u64) + TTBR1_EL1::BADDR.val(ttb1 >> 1));

    debug!(
        "enable_paging: {:x}, {:x}, {}, {:x}",
        TTBR0_EL1.get(),
        TTBR1_EL1.get(),
        asid,
        offset
    );

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

    unsafe {
        barrier::isb(barrier::SY);

        asm!("add sp, sp, {0}", in(reg) offset);
    }

    debug!("through!!!");

    Ok(())
}

pub fn set_vbar(_translation: impl Translate) -> Result<()> {
    use crate::pager::Identity;
    use cortex_a::regs::*;

    extern "C" {
        static vector_table_el1: u8;
    }
    let virt_addr =
        unsafe { Identity::new().translate_phys(PhysAddr::from_linker_symbol(&vector_table_el1))? };

    VBAR_EL1.set(virt_addr.get() as u64);

    Ok(())
}
