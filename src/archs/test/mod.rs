// SPDX-License-Identifier: Unlicense

//! Dead-bat implementation of an architecture to allow tests to compile.

#![allow(missing_docs)]

use crate::pager::{
    Addr, AddrRange, Attributes, FixedOffset, FrameAllocator, HandlerReturnAction, PhysAddr,
    PhysAddrRange, Translate, VirtAddr, VirtAddrRange,
};
use crate::util::locked::Locked;
use crate::Result;

pub struct Arch {}

#[allow(unused_variables)]
impl super::PagerTrait for Arch {
    fn ram_range() -> Result<PhysAddrRange> {
        Ok(PhysAddrRange::between(
            PhysAddr::at(0x4000_0000),
            PhysAddr::at(0x8000_0000),
        ))
    }

    fn kernel_base() -> VirtAddr {
        VirtAddr::at(0x1_0000_0000_0000)
    }

    fn kernel_offset() -> FixedOffset {
        FixedOffset::new(PhysAddr::at(0x4000_0000), VirtAddr::at(0x1_4000_0000))
    }

    fn boot_image() -> PhysAddrRange {
        PhysAddrRange::between(PhysAddr::at(0x4008_0000), PhysAddr::at(0x400b_d000))
    }

    fn text_image() -> PhysAddrRange {
        PhysAddrRange::new(PhysAddr::at(0x4008_0000), 0x1_8000)
    }

    fn static_image() -> PhysAddrRange {
        PhysAddrRange::new(PhysAddr::at(0x4009_8000), 0x1000)
    }

    fn bss_image() -> PhysAddrRange {
        unimplemented!()
    }

    fn data_image() -> PhysAddrRange {
        PhysAddrRange::new(PhysAddr::at(0x4009_9000), 0x1000)
    }

    fn stack_range() -> PhysAddrRange {
        PhysAddrRange::new(PhysAddr::at(0x4040_0000), 0x8000)
    }

    fn pager_init() -> Result<()> {
        Ok(())
    }

    fn enable_paging(page_directory: &impl super::PageDirectory) -> Result<()> {
        unimplemented!()
    }

    fn move_stack(stack_pointer: VirtAddr, next: fn() -> !) -> ! {
        unimplemented!()
    }
}

pub struct PageBlockDescriptor(u64);

pub struct PageDirectory {}

impl PageDirectory {
    pub const fn new() -> Self {
        Self {}
    }
}

#[allow(unused_variables)]
impl super::PageDirectory for PageDirectory {
    fn as_any(&self) -> &dyn Any {
        unimplemented!()
    }

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

    fn maps_to(
        &self,
        virt_addr: VirtAddr,
        mem_access_translation: &FixedOffset,
    ) -> Result<PhysAddr> {
        unimplemented!()
    }

    fn unmap(
        &mut self,
        virt_addr_range: VirtAddrRange,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &FixedOffset,
    ) -> Result<()> {
        unimplemented!()
    }

    fn dump(&self, mem_access_translation: &impl Translate) {
        unimplemented!()
    }
}

pub fn new_page_directory() -> impl super::PageDirectory {
    PageDirectory {}
}

impl super::DeviceTrait for Arch {
    fn add_handler(_interrupt: u8, _handler: fn() -> HandlerReturnAction) -> Result<()> {
        unimplemented!()
    }

    fn debug_uart() -> Result<PhysAddrRange> {
        unimplemented!()
    }
}

impl super::HandlerTrait for Arch {
    fn handler_init() -> Result<()> {
        Ok(())
    }
}

#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    unreachable!()
}

use crate::pager::Page;
use core::any::Any;

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

#[inline(always)]
pub fn core_id() -> u8 {
    1
}
