use super::PageRange;
use crate::pager::{Page, PAGESIZE_BYTES};
use crate::util::locked::Locked;

use core::fmt::{Debug, Error, Formatter};
use core::ptr;

pub struct PageBumpAllocator {
    limit: usize,
    top: *const Page,
}

unsafe impl Sync for Locked<PageBumpAllocator> {}

impl PageBumpAllocator {
    pub const fn new() -> Self {
        Self {
            limit: 0,
            top: ptr::null(),
        }
    }

    pub fn reset(&mut self, range: PageRange) {
        self.limit = range.length_in_pages();
        self.top = range.top();
    }

    pub fn alloc(&mut self, span: usize) -> Result<*const Page, u64> {
        if span > self.limit {
            Err(0)
        } else {
            self.limit -= span;
            self.top = unsafe { self.top.offset(-1 * (span as isize)) };
            Ok(self.top)
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
