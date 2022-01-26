// SPDX-License-Identifier: Unlicense

//! Responding to virtual memory exceptions

use super::{
    frames, mem_translation, Addr, AddrRange, AttributeField, Attributes, FixedOffset,
    FrameAllocator, FramePurpose, VirtAddr, VirtAddrRange, KERNEL_PAGE_DIRECTORY,
};

use crate::archs::{arch::Arch, PageDirectory, PagerTrait};
use crate::Result;

/// What the architecture should do after a handler invocation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HandlerReturnAction {
    /// Resume the thread running at the time of the exception  
    Return,
    /// Prepare to suspend the process and yield to the scheduler
    Yield,
}

/// The kernel has accessed an invalid page.
///
/// FIXME: is this called when fault_addr is outside of kernel_range?
pub fn kernel_translation_fault(
    fault_addr: VirtAddr,
    _level: Option<u64>,
) -> Result<HandlerReturnAction> {
    info!("kernel_translation_fault: {:?}", fault_addr);

    assert_gt!(fault_addr, Arch::kernel_base());
    let phys_addr = frames::allocator()
        .lock()
        .alloc_zeroed(FramePurpose::Kernel)?;
    let translation = FixedOffset::new(phys_addr, fault_addr.page_base());
    const ATTRIBUTES: Attributes = Attributes::KERNEL_DATA.set(AttributeField::Accessed);
    let mut page_directory = KERNEL_PAGE_DIRECTORY.lock();
    page_directory.map_translation(
        VirtAddrRange::page_containing(fault_addr),
        translation,
        ATTRIBUTES,
        frames::allocator(),
        mem_translation(),
    )?;
    Ok(HandlerReturnAction::Return)
}
