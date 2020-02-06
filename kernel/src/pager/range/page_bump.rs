use crate::util::locked::Locked;

use crate::pager::virt_addr::{VirtAddr, VirtAddrRange};
use crate::pager::PAGESIZE_BYTES;
use core::fmt::{Debug, Error, Formatter};

pub struct PageBumpAllocator {
    limit: usize,
    top: VirtAddr,
}

unsafe impl Sync for Locked<PageBumpAllocator> {}

impl PageBumpAllocator {
    pub const fn new() -> Self {
        Self {
            limit: 0,
            top: VirtAddr::new(0),
        }
    }

    pub fn reset(&mut self, range: VirtAddrRange) {
        self.limit = range.length_in_pages();
        self.top = range.top();
    }

    pub fn alloc(&mut self, span: usize) -> Result<VirtAddrRange, u64> {
        if span > self.limit {
            Err(0)
        } else {
            let length = span * PAGESIZE_BYTES;
            self.limit -= span;
            self.top = self.top.decrement(length);
            Ok(VirtAddrRange::new(self.top, length))
        }
    }
}

impl Debug for PageBumpAllocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "PageBumpAllocator{{ limit: {}, top: {:?} }}",
            self.limit, self.top
        )
    }
}
