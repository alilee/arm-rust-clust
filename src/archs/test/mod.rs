// SPDX-License-Identifier: Unlicense

//! Dead-bat implementation of an architecture to allow tests to compile.

use crate::pager::{
    Attributes, FrameAllocator, PhysAddr, PhysAddrRange, Translate, VirtAddr, VirtAddrRange,
};
use crate::util::locked::Locked;
use crate::Result;

pub struct Arch {}

#[allow(unused_variables)]
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

    fn enable_paging(page_directory: &impl super::PageDirectory) {}

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

pub struct PageDirectory {}

#[allow(unused_variables)]
impl super::PageDirectory for PageDirectory {
    fn map_translation(
        &mut self,
        phys_range: PhysAddrRange,
        virtual_address_translation: impl Translate,
        attrs: Attributes,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &impl Translate,
    ) {
        unimplemented!()
    }

    fn map_demand(
        &mut self,
        virtual_range: VirtAddrRange,
        attrs: Attributes,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &impl Translate,
    ) {
        unimplemented!()
    }
}

pub fn new_page_directory() -> impl super::PageDirectory {
    PageDirectory {}
}

#[no_mangle]
pub unsafe extern "C" fn _reset() -> ! {
    unreachable!()
}

#[no_mangle]
pub static image_base: u8 = 0u8;

#[no_mangle]
pub static image_end: u8 = 0u8;
