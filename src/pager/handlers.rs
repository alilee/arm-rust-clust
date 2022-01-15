// SPDX-License-Identifier: Unlicense

//! Responding to virtual memory exceptions

use super::{frames, mem_translation, AttributeField, Attributes, VirtAddr, KERNEL_PAGE_DIRECTORY};

use crate::archs::{arch::Arch, PageDirectory, PagerTrait};
use crate::Result;

/// The kernel has accessed an invalid page.
///
/// FIXME: what if fault_addr is outside of kernel_range?
pub fn kernel_translation_fault(fault_addr: VirtAddr, _level: Option<u64>) -> Result<()> {
    info!("kernel_translation_fault: {:?}", fault_addr);
    assert_gt!(fault_addr, Arch::kernel_base());
    const ATTRIBUTES: Attributes = Attributes::KERNEL_DATA.set(AttributeField::Accessed);
    KERNEL_PAGE_DIRECTORY.lock().demand_page(
        fault_addr,
        ATTRIBUTES,
        frames::allocator(),
        mem_translation(),
    )
}
