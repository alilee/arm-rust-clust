//! Manages virtual address range assignments. 
//!
//! Allows a process to receive a block of address ranges which has not previously been requested,
//! and define some access characteristics.  

use core::mem::transmute;

/// A single allocated or free address range.
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub base_page: u32,
    n_pages: i32,  // negative if allocated
}

impl Range {

    /// A null range as placeholder for testing
    pub fn null() -> Range {
        Range { base_page: 0, n_pages: 0 }
    }

    /// An available range covering entire address space
    pub fn all_free() -> Range {
        Range { base_page: 0, n_pages: (!0u32 / super::super::PAGESIZE_BYTES) as i32 }
    }
    
    /// Not allocated and is larger than specified number of pages 
    pub fn available_for(&self, n_pages: u8) -> bool {
        self.n_pages > n_pages as i32
    }
    
    /// Return a new free range less n_pages from the beginning    
    pub fn residual_after(&self, n_pages: u8) -> Range {
        Range { base_page: self.base_page + n_pages as u32, n_pages: self.n_pages - n_pages as i32 }
    }
    
    /// Update to be allocated with new length
    pub fn allocate(&mut self, n_pages: u8) {
        self.n_pages = -(n_pages as i32); 
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_init() {
        assert_eq!(1, 1);
    }
    
}