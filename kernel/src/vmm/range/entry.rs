//! Manages virtual address range assignments.
//!
//! Allows a process to receive a block of address ranges which has not previously been requested,
//! and define some access characteristics.

/// A single allocated or free address range.
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub base_page: u32,
    n_pages: i32, // negative if allocated
}

pub const RANGE_SIZE: usize = 8;

#[allow(dead_code)]
impl Range {
    /// A null range as placeholder for testing
    pub fn null() -> Range {
        Range {
            base_page: 0,
            n_pages: 0,
        }
    }

    /// An available range covering entire address space
    pub fn all_free() -> Range {
        Range {
            base_page: 0,
            n_pages: (!0u32 / super::super::PAGESIZE_BYTES) as i32,
        }
    }

    /// Not allocated and is larger than specified number of pages
    pub fn available_for(&self, n_pages: u8) -> bool {
        self.n_pages > n_pages as i32
    }

    /// Not allocated and completely overlays the specified page range
    pub fn available_over(&self, page: u32, n_pages: u8) -> bool {
        self.n_pages > 0 && self.base_page <= page &&
            (self.base_page + self.n_pages as u32) >= (page + n_pages as u32)
    }

    /// Return a new free range less n_pages from the beginning
    pub fn residual_after(&self, n_pages: u8) -> Option<Range> {
        let back = self.n_pages - n_pages as i32;
        if back > 0 {
            Some(Range {
                base_page: self.base_page + n_pages as u32,
                n_pages: back,
            })
        } else {
            None
        }
    }

    /// Return a new free range for the part of the free area before the specified range
    pub fn residual_front(&self, page: u32, _: u8) -> Option<Range> {
        if page > self.base_page {
            Some(Range {
                base_page: self.base_page,
                n_pages: (page - self.base_page) as i32,
            })
        } else {
            None
        }
    }

    /// Return a new free range for the part of the free area after the specified range
    pub fn residual_back(&self, page: u32, n_pages: u8) -> Option<Range> {
        let back = self.n_pages - (n_pages as i32) - ((page - self.base_page) as i32);
        if back > 0 {
            Some(Range {
                base_page: page + n_pages as u32,
                n_pages: back,
            })
        } else {
            None
        }
    }

    /// Update to be allocated with new length
    pub fn allocate(&mut self, n_pages: u8) {
        self.n_pages = -(n_pages as i32);
    }

    /// Update to be allocated with new length and page
    pub fn allocate_fixed(&mut self, page: u32, n_pages: u8) {
        self.allocate(n_pages);
        self.base_page = page;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use core::mem;

    #[test]
    fn test_init() {
        // assert_eq!(mem::size_of::<Range>(), 8);
        //
        // assert_eq!(Range::all_free().base_page, 0);
        // assert_eq!(Range::all_free().n_pages, (!0u32 / 4096) as i32);
        // assert_eq!(Range::all_free().available_for(1), true);
        // assert_eq!(Range::all_free().available_for(255), true);
        //
        // let mut r = Range::all_free();
        // r.allocate(6);
        // assert_eq!(r.available_for(3), false);
        // assert_eq!(Range::all_free().residual_after(6).unwrap().available_for(3), true);
        //
        // let mut r = Range::all_free();
        // assert_eq!(r.available_over(0, 3), true);
        // assert_eq!(r.available_over(3, 3), true);
        // assert_eq!(r.available_over((!0u32 / 4096), 3), false);
        // r.allocate_fixed(3,3);
        // assert_eq!(r.available_for(1), false);
    }

}
