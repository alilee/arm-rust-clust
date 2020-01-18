//! Manages a page frame table, which tracks which pages of physical memory
//! have been allocated for virtual address ranges
//!
//! Pages which have not been allocated, are free memory. Since pages may
//! be freed, there may be fragmentation, if multiple pages are required.

use super::{PhysAddr, PhysAddrRange};

const MAX_MEMORY: usize = (258 * 1024 * 1024);
const PAGE_SIZE: usize = 4096;
const MAX_FRAMES: usize = MAX_MEMORY / PAGE_SIZE;

/// The frame table is the highest currently used page and a stack of the free pages
/// residing under that mark
pub struct FrameTable {
    page_map: [u64; MAX_FRAMES / 64], // bitmap of pages where 0 is available and 1 is allocated
    highwater_mark: usize,            // index of lowest page map entry that has any
    range: PhysAddrRange,             // range of physical memory we can allocate
}

#[allow(dead_code)]
impl FrameTable {
    pub const fn init() -> Self {
        FrameTable {
            page_map: [0; MAX_FRAMES / 64],
            highwater_mark: 0,
            range: PhysAddrRange {
                base: PhysAddr(0),
                length: 0,
            },
        }
    }

    /// Initialise the page frame data structure into a specific physical address
    pub fn reset(self: &mut Self, range: PhysAddrRange) -> Result<(), u64> {
        for x in self.page_map.iter_mut() {
            *x = 0
        }
        self.range = range;
        self.highwater_mark = 0;
        Ok(())
    }

    pub fn reserve(self: &mut Self, range: PhysAddrRange) -> Result<(), u64> {
        if range.outside(&self.range) {
            return Err(0);
        }
        let first_page = PhysAddrRange::bounded_by(self.range.base, range.base).pages(PAGE_SIZE);
        let mut i = first_page / 64;
        let mut offset = first_page % 64;
        let mut n_pages_reqd = range.pages(PAGE_SIZE);
        while n_pages_reqd > 0 {
            let chunk = self.page_map[i];
            let pages = core::cmp::min(n_pages_reqd, 64 - offset);
            let reserve = ((1 << (pages + 1) - 1) << offset) as u64;
            if chunk & reserve > 0 {
                return Err(0);
            }
            self.page_map[i] = chunk | reserve;
            n_pages_reqd -= pages;
            i += 1;
        }
        Ok(())
    }

    pub fn find_and_reserve(self: &mut Self, len: usize) -> Result<PhysAddr, u64> {
        todo!()
    }

    pub fn free(self: &mut Self, base: PhysAddr, len: usize) -> Result<(), u64> {
        todo!()
    }

    pub fn print_state(self: &mut Self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(true);
    }
}
