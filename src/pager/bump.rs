// SPDX-License-Identifier: Unlicense

//! Allocate pages within a range, without dealloc.
//!
//! Suited for device MMIO range.

use super::{Addr, AddrRange, VirtAddr, VirtAddrRange, PAGESIZE_BYTES};
use crate::Result;

use core::fmt::{Debug, Formatter};

/// An allocator of pages within a single range.
pub struct PageBumpAllocator {
    limit: usize,
    top: VirtAddr,
}

// unsafe impl Sync for Locked<PageBumpAllocator> {}

impl PageBumpAllocator {
    /// Empty allocator.
    ///
    /// Pop the pages off the top of a stack until limit reached.
    pub const fn new() -> Self {
        Self {
            limit: 0,
            top: VirtAddr::null(),
        }
    }

    /// Add range to allocator.
    pub fn reset(&mut self, range: VirtAddrRange) -> Result<()> {
        self.limit = range.length_in_pages();
        self.top = range.top();
        Ok(())
    }

    /// Allocate a number of pages from the pool range.
    pub fn alloc(&mut self, pages: usize) -> Result<VirtAddrRange> {
        info!("allocating {} pages", pages);
        if pages > self.limit {
            Err(crate::Error::OutOfMemory)
        } else {
            let length = pages * PAGESIZE_BYTES;
            self.limit -= pages;
            self.top = self.top.decrement(length);
            Ok(VirtAddrRange::new(self.top, length))
        }
    }
}

impl Debug for PageBumpAllocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PageBumpAllocator{{ limit: {}, top: {:?} }}",
            self.limit, self.top
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc() {
        let mut allocator = PageBumpAllocator::new();
        assert_err!(allocator.alloc(1));

        let base = VirtAddr::at(0xd000);
        let virt_addr_range = VirtAddrRange::new(base, 0x3000);
        assert_ok!(allocator.reset(virt_addr_range));

        assert_ok_eq!(allocator.alloc(1), VirtAddrRange::new(VirtAddr::at(0xf000), 0x1000));
        assert_ok_eq!(allocator.alloc(2), VirtAddrRange::new(VirtAddr::at(0xd000), 0x2000));
        assert_err!(allocator.alloc(1));
    }
}