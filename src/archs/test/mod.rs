// SPDX-License-Identifier: Unlicense

//! Dead-bat implementation of an architecture to allow tests to compile.

use crate::pager::{
    Addr, AddrRange, Attributes, FrameAllocator, PhysAddr, PhysAddrRange, Translate, VirtAddr,
    VirtAddrRange,
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
        virt_addr_range: VirtAddrRange,
        translation: impl Translate,
        attributes: Attributes,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &impl Translate,
    ) -> Result<VirtAddrRange> {
        unimplemented!()
    }

    fn dump(&self, mem_access_translation: &impl Translate) {
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

use crate::pager::Page;

#[no_mangle]
pub static text_base: Page = Page::new();

#[no_mangle]
pub static text_end: Page = Page::new();

#[no_mangle]
pub static static_base: Page = Page::new();

#[no_mangle]
pub static static_end: Page = Page::new();

#[no_mangle]
pub static data_base: Page = Page::new();

#[no_mangle]
pub static data_end: Page = Page::new();
