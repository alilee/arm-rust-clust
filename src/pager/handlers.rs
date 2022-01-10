// SPDX-License-Identifier: Unlicense

//! Responding to virtual memory exceptions

use crate::archs::{arch::Arch, PageDirectory, PagerTrait};
use crate::pager::VirtAddr;
use crate::Result;

use super::frames;
use super::mem_translation;
use super::KERNEL_PAGE_DIRECTORY;

/// The kernel has accessed an invalid page.
pub fn kernel_translation_fault(fault_addr: VirtAddr, _level: Option<u64>) -> Result<()> {
    info!("kernel_translation_fault: {:?}", fault_addr);
    assert_gt!(fault_addr, Arch::kernel_base());
    KERNEL_PAGE_DIRECTORY
        .lock()
        .demand_page(fault_addr, frames::allocator(), mem_translation())
}
