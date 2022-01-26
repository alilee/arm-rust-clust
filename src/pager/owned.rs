// SPDX-License-Identifier: Unlicense

//! A handle for owned kernel-mapped pages which should be freed on Drop.

use super::{frames, mem_fixed_offset, AddrRange, VirtAddr, VirtAddrRange, KERNEL_PAGE_DIRECTORY};

use crate::archs::PageDirectory;

/// An owned mapping which will be unmapped on Drop.
#[derive(Debug, Clone)]
pub struct OwnedMapping {
    virt_addr_range: VirtAddrRange,
}

impl OwnedMapping {
    /// Make a virtual address range subject to unmapping on Drop.
    pub fn new(virt_addr_range: VirtAddrRange) -> Self {
        Self { virt_addr_range }
    }

    /// Get the base address of the range.
    pub fn base(&self) -> VirtAddr {
        self.virt_addr_range.base()
    }

    /// Get the range.
    pub fn range(&self) -> VirtAddrRange {
        self.virt_addr_range
    }

    /// Get the length of the range, in bytes.
    pub fn length(&self) -> usize {
        self.virt_addr_range.length()
    }
}

impl Drop for OwnedMapping {
    fn drop(&mut self) {
        let mut page_directory = KERNEL_PAGE_DIRECTORY.lock();
        page_directory
            .unmap(
                self.virt_addr_range,
                frames::allocator(),
                mem_fixed_offset(),
            )
            .expect("PageDirectory::unmap");
    }
}
