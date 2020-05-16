mod attrs;
mod desc;
mod mair;
mod mapper;
mod table;
mod trans;

use crate::pager::{attrs::Attributes, virt_addr::*, MemOffset, PhysAddr, PhysAddrRange};
use mapper::Mapper;
use table::{Translation, TranslationAttributes};

use log::{debug, info};

pub const KERNEL_BASE: VirtAddr = VirtAddr::new_const(table::UPPER_VA_BASE);
pub const USER_TOP: VirtAddr = VirtAddr::new_const(table::LOWER_VA_TOP);

pub fn init() -> Result<(), u64> {
    info!("init");
    table::init()
}

pub fn enable(kernel_offset: AddrOffsetUp) -> ! {
    info!("enable");

    let ttbr1 = Translation::ttbr1();
    let ttbr0 = Translation::ttbr0();

    enable_paging(ttbr1, ttbr0, 0);

    // in one asm, so that SP can't move between get and set
    unsafe {
        let offset = kernel_offset.get_offset();
        llvm_asm!("add sp, sp, $0"
             :
             : "r"(offset));
    }
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
    let attributes = TranslationAttributes::from_attrs(attributes);
    let virt_range = VirtAddrRange::id_map(phys_range);
    Translation::map_to(virt_range, Mapper::identity(), attributes, mem_offset)
}

pub fn absolute_map(
    phys_range: PhysAddrRange,
    virt_base: VirtAddr,
    attributes: Attributes,
    mem_offset: MemOffset,
) -> Result<VirtAddrRange, u64> {
    let attributes = TranslationAttributes::from_attrs(attributes);
    let offset_mapper = Mapper::reverse_translation(phys_range.base(), virt_base);
    Translation::map_to(virt_range, offset_mapper, attributes, mem_offset)
}

pub fn demand_map(
    virt_range: VirtAddrRange,
    attributes: Attributes,
    mem_offset: MemOffset,
) -> Result<VirtAddrRange, u64> {
    let attributes = TranslationAttributes::from_attrs_provisional(attributes);
    Translation::map_to(virt_range, Mapper::demand(), attributes, mem_offset)
}

pub fn fulfil_map(
    virt_range: VirtAddrRange,
    attributes: Attributes,
    mem_offset: MemOffset,
) -> Result<VirtAddrRange, u64> {
    let attributes = TranslationAttributes::from_attrs(attributes);
    Translation::map_to(virt_range, Mapper::fulfil(), attributes, mem_offset)
}

pub fn user_tt_page() -> Result<PhysAddr, u64> {
    Ok(PhysAddr::new(Translation::ttbr0() as usize))
}
