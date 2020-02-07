mod attrs;
mod desc;
mod mair;
mod table;
mod trans;

use crate::pager::{range::attrs::Attributes, virt_addr::*, MemOffset, PhysAddrRange};
use table::{Translation, TranslationAttributes};

use log::{debug, info};

pub const KERNEL_BASE: VirtAddr = VirtAddr::new_const(table::UPPER_VA_BASE);

pub fn init() -> Result<(), u64> {
    info!("init");
    table::init()
}

pub fn enable(boot3: fn() -> !, kernel_offset: AddrOffsetUp) -> ! {
    info!("enable boot3@{:?}", boot3);

    let ttbr1 = Translation::ttbr1();
    let ttbr0 = Translation::ttbr0();

    enable_paging(ttbr1, ttbr0, 0);

    // in one asm, so that SP can't move between get and set
    unsafe {
        let offset = kernel_offset.get_offset();
        asm!("add sp, sp, $0"
             :
             : "r"(offset));
    }

    boot3()
}

fn enable_paging(ttbr1: u64, ttbr0: u64, asid: u16) {
    use cortex_a::{
        barrier,
        regs::{SCTLR_EL1::*, *},
    };

    debug!(
        "enable_paging(ttrb1: {:#x}, ttbr0: {:#x}, asid: {:#x})",
        ttbr1, ttbr0, asid
    );

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
    let mut tt = if VirtAddr::id_map(phys_range.base()) < KERNEL_BASE {
        Translation::tt0(mem_offset).unwrap()
    } else {
        Translation::tt1(mem_offset).unwrap()
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
    let mut tt = if virt_base < KERNEL_BASE {
        Translation::tt0(mem_offset).unwrap()
    } else {
        Translation::tt1(mem_offset).unwrap()
    };
    let attributes = TranslationAttributes::from(attributes);
    tt.absolute_map(phys_range, virt_base, attributes)
}

pub fn provisional_map(
    virt_range: VirtAddrRange,
    attributes: Attributes,
    mem_offset: MemOffset,
) -> Result<VirtAddrRange, u64> {
    let mut tt = if virt_range.base() < KERNEL_BASE {
        Translation::tt0(mem_offset).unwrap()
    } else {
        Translation::tt1(mem_offset).unwrap()
    };
    let attributes = TranslationAttributes::from(attributes);
    tt.provisional_map(virt_range, attributes)
}
