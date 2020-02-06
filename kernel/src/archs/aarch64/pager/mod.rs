mod table;

use crate::pager::{range::attrs::Attributes, virt_addr::*, MemOffset, PhysAddrRange};
use table::{Translation, TranslationAttributes};

use log::{debug, info};

pub const KERNEL_BASE: VirtAddr = VirtAddr::new(table::UPPER_VA_BASE);

pub fn init() -> Result<(), u64> {
    info!("init");
    table::init();

    Ok(())
}

pub fn enable(boot3: fn() -> !, offset: XVirtOffset) -> ! {
    info!("enable boot3@{:?}", boot3);

    let ttbr1 = Translation::ttbr1();
    let ttbr0 = Translation::ttbr0();

    enable_paging(ttbr1, ttbr0, 0);

    // in one asm, so that SP can't move between get and set
    unsafe {
        asm!("add sp, sp, $0"
             :
             : "r"(offset.get()));
    }

    boot3()
}

fn enable_paging(ttbr1: u64, ttbr0: u64, asid: u16) {
    use cortex_a::{
        barrier,
        regs::{SCTLR_EL1::*, TCR_EL1::*, *},
    };

    debug!(
        "enable_paging(ttrb1: {:#x}, ttbr0: {:#x}, asid: {:#x})",
        ttbr1, ttbr0, asid
    );

    let ttbr0: u64 = ttbr0 | ((asid as u64) << 48);
    TTBR0_EL1.set(ttbr0);
    TTBR1_EL1.set(ttbr1);

    assert_eq!(crate::pager::PAGESIZE_BYTES, 4096);
    //
    // TODO: nTWE, nTWI
    //
    TCR_EL1.modify(
        AS::Bits_16    // 16 bit ASID 
            + IPS::Bits_36  // 36 bits/64GB of physical address space
            + TG1::KiB_4
            + SH1::Outer
            + ORGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T1SZ.val(64 - table::kernel_va_bits() as u64) // 64-t1sz=43 bits of address space in high range
            + TG0::KiB_4
            + SH0::Outer
            + ORGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T0SZ.val(64 - table::user_va_bits() as u64), // 64-t0sz=48 bits of address space in low range
    );
    unsafe {
        barrier::isb(barrier::SY);
    }

    SCTLR_EL1.modify(I::SET + C::SET + M::SET);
    unsafe {
        barrier::isb(barrier::SY);
    }
}

pub fn identity_map(
    phys_range: PhysAddrRange,
    attributes: Attributes,
    mem_offset: MemOffset,
) -> Result<VirtAddrRange, u64> {
    let mut tt1 = Translation::tt1(mem_offset).unwrap();
    let mut tt0 = Translation::tt0(mem_offset).unwrap();

    let mut tt = if VirtAddr::id_map(phys_range.base()) < KERNEL_BASE {
        tt0
    } else {
        tt1
    };
    let attributes = TranslationAttributes::from(attributes);
    tt.identity_map(phys_range, attributes)
}

pub fn absolute_map(
    phys_range: PhysAddrRange,
    virt_base: VirtAddr,
    attributes: Attributes,
    mem_offset: MemOffset,
) -> Result<VirtAddrRange, u64> {
    let mut tt1 = Translation::tt1(mem_offset).unwrap();
    let mut tt0 = Translation::tt0(mem_offset).unwrap();

    let mut tt = if virt_base < KERNEL_BASE { tt0 } else { tt1 };
    let attributes = TranslationAttributes::from(attributes);
    tt.absolute_map(phys_range, virt_base, attributes)
}
