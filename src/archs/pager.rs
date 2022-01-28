// SPDX-License-Identifier: Unlicense

//! Interface for paging functions.

use crate::pager::{
    Attributes, FixedOffset, FrameAllocator, PhysAddr, PhysAddrRange, Translate, VirtAddr,
    VirtAddrRange,
};
use crate::util::locked::Locked;
use crate::Result;

use core::any::Any;

/// Each architecture must supply the following entry points for paging..
pub trait PagerTrait {
    /// Physical address range of ram
    fn ram_range() -> PhysAddrRange;
    /// Base virtual address of kernel address space
    fn kernel_base() -> VirtAddr;

    /// Kernel offset on boot
    fn kernel_offset() -> FixedOffset;
    /// Kernel boot image
    fn boot_image() -> PhysAddrRange;
    /// Kernel code
    fn text_image() -> PhysAddrRange;
    /// Kernel read-only data
    fn static_image() -> PhysAddrRange;
    /// Kernel zero-initialised
    fn bss_image() -> PhysAddrRange;
    /// Kernel dynamic data (includes bss)
    fn data_image() -> PhysAddrRange;
    /// Kernel reset stack
    fn stack_range() -> PhysAddrRange;

    /// Initialise virtual memory management.
    fn pager_init() -> Result<()>;
    /// Enable virtual memory management.
    fn enable_paging(page_directory: &impl PageDirectory) -> Result<()>;
    /// Move the stack pointer and branch
    fn move_stack(stack_pointer: VirtAddr, next: fn() -> !) -> !;
}

/// Methods to maintain a directory of virtual to physical addresses.
pub trait PageDirectory {
    /// Enable downshift to arch-specific concrete page directories.
    fn as_any(&self) -> &dyn Any;

    /// Map physical address range at offset.
    fn map_translation(
        &mut self,
        virt_addr_range: VirtAddrRange,
        translation: impl Translate + core::fmt::Debug,
        attributes: Attributes,
        allocator: &Locked<impl FrameAllocator>,
        mem_access_translation: &impl Translate,
    ) -> Result<VirtAddrRange>;

    /// Return the current physical address for a virtual address
    fn maps_to(
        &self,
        virt_addr: VirtAddr,
        mem_access_translation: &FixedOffset,
    ) -> Result<PhysAddr>;

    /// Unmap a previously mapped range, and return any memory to the allocator.
    fn unmap(
        &mut self,
        virt_addr_range: VirtAddrRange,
        allocator: &'static Locked<impl FrameAllocator>,
        mem_access_translation: &FixedOffset,
    ) -> Result<()>;

    /// Log the state of the page directory at debug.
    fn dump(&self, mem_access_translation: &impl Translate);
}

/// Construct an empty page directory.
/// TODO: Should this be in Arch trait? limitation of generics in traits right now.
pub fn new_page_directory() -> impl PageDirectory {
    super::arch::new_page_directory()
}
