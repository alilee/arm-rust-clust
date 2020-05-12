// SPDX-License-Identifier: Unlicense

//! Dead-bat implementation of an architecture to allow tests to compile.

use crate::pager::{
    Attributes, FrameAllocator, PhysAddr, PhysAddrRange, Translate, VirtAddr, VirtAddrRange,
};
use crate::util::locked::Locked;
use crate::Result;

pub struct Arch {}

impl super::ArchTrait for Arch {
    fn ram_range() -> Result<PhysAddrRange> {
        Ok(PhysAddrRange::between(
            PhysAddr::at(0x4000_0000),
            PhysAddr::at(0x8000_0000),
        ))
    }

    fn kernel_base() -> VirtAddr {
        VirtAddr::at(0x1_0000_0000_0000)
    }

    fn pager_init() -> Result<()> {
        Ok(())
    }

    fn map_translation(
        _phys_range: PhysAddrRange,
        _virtual_address_translation: impl Translate,
        _attrs: Attributes,
        _allocator: &Locked<impl FrameAllocator>,
        _mem_access_translation: impl Translate,
    ) {
    }

    fn map_demand(
        _virtual_range: VirtAddrRange,
        _attrs: Attributes,
        _allocator: &Locked<impl FrameAllocator>,
        _mem_access_translation: impl Translate,
    ) {
    }

    fn enable_paging() {}

    fn handler_init() -> Result<()> {
        Ok(())
    }

    fn thread_init() -> Result<()> {
        Ok(())
    }

    fn wait_forever() -> ! {
        loop {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn _reset() -> ! {
    unreachable!()
}

#[no_mangle]
pub static image_base: u8 = 0u8;

#[no_mangle]
pub static image_end: u8 = 0u8;
