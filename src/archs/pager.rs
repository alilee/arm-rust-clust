// SPDX-License-Identifier: Unlicense

//! Interface for paging functions.

use crate::pager::{
    Addr, AddrRange, Attributes, FrameAllocator, PhysAddrRange, Translate, VirtAddr, VirtAddrRange,
};
use crate::util::locked::Locked;
use crate::Result;

/// Each architecture must supply the following entry points for paging..
pub trait PagerTrait {
    /// Physical address range of ram
    fn ram_range() -> Result<PhysAddrRange>;
    /// Base virtual address of kernel address space
    fn kernel_base() -> VirtAddr;

    /// Initialise virtual memory management.
    fn pager_init() -> Result<()>;
    /// Enable virtual memory management.
    fn enable_paging(page_directory: &impl PageDirectory);
}

/// Methods to maintain a directory of virtual to physical addresses.
pub trait PageDirectory {
    /// Map physical address range at offset.
    fn map_translation(
        &mut self,
        virt_addr_range: VirtAddrRange,
        translation: impl Translate + core::fmt::Debug,
        attributes: Attributes,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &impl Translate,
    ) -> Result<VirtAddrRange>;

    /// Log the state of the page directory at debug.
    fn dump(&self, mem_access_translation: &impl Translate);
}

/// Construct an empty page directory.
/// TODO: Should this be in Arch trait? limitation of generics in traits right now.
pub fn new_page_directory() -> impl PageDirectory {
    super::arch::new_page_directory()
}
