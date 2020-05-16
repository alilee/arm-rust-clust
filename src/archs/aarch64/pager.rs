// SPDX-License-Identifier: Unlicense

//! Paging trait for aarch64

use crate::pager::{Attributes, FrameAllocator, PhysAddr, PhysAddrRange, Translate, VirtAddrRange};
use crate::util::locked::Locked;

/// Aarch64 implementation of a page directory
pub struct PageDirectory {
    ttb0: Option<PhysAddr>, // physical address of the root table for user space
    ttb1: Option<PhysAddr>, // physical address of the root table for kernel space
}

#[allow(unused_variables)]
impl crate::archs::PageDirectory for PageDirectory {
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

impl PageDirectory {
    fn new() -> Self {
        Self {
            ttb0: None,
            ttb1: None,
        }
    }
}

pub fn new_page_directory() -> impl crate::archs::PageDirectory {
    PageDirectory::new()
}
