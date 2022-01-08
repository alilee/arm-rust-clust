// SPDX-License-Identifier: Unlicense

//! Responding to virtual memory exceptions

use crate::pager::VirtAddr;
use crate::Result;
use crate::archs::{PageDirectory, PagerTrait, arch::Arch};

use super::KERNEL_PAGE_DIRECTORY;
use super::frames;
use super::mem_translation;

/// The kernel has accessed an invalid page.
pub fn kernel_translation_fault(fault_addr: VirtAddr, _level: Option<u64>) -> Result<()> {
    assert_gt!(fault_addr, Arch::kernel_base());
    let mut page_dir =
        KERNEL_PAGE_DIRECTORY.lock();
    page_dir
        .demand_page(
            fault_addr,
            frames::allocator(),
            mem_translation(),
        ).expect("Kernel accessing unmapped ranges");
    Ok(())
}