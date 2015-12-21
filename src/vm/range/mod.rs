//! Manages virtual address range assignments. 
//!
//! Allows a process to receive a block of address ranges which has not previously been requested,
//! and define some access characteristics.  

use core::mem::transmute;

mod entry;

/// The range table is a stack of address ranges
pub struct Table {
    ranges: [entry::Range; (super::PAGESIZE_WORDS as usize - 1) / 2],
    n_ranges: usize,            // offset of top of stack 
}

impl Table {
    
    /// Initialise the address range data structure into a specific physical address
    pub fn init<'a>(a: *mut u32) -> &'a mut Table {
        unsafe {
            let p_table = transmute::<*mut u32, &mut Table>(a); 
            p_table.n_ranges = 1;
            p_table.ranges[0] = entry::Range::all_free();
            p_table
        }
    }
    
    fn push(&mut self, r: entry::Range) {
        self.ranges[self.n_ranges] = r;
        self.n_ranges += 1;
    }
    
    /// Request an address range of a specified number of pages in length
    pub fn request(&mut self, n_pages: u8) -> Option<u32> {
        for i in 0..self.n_ranges {
            if self.ranges[i].available_for(n_pages) {
                // TODO: check for out of range entries
                let r = self.ranges[i];
                self.push(r.residual_after(n_pages));
                self.ranges[i].allocate(n_pages);
                return Some(self.ranges[i].base_page);
            }
        }
        None
    }

    /// Request an address range of a specified length at a specified address
    pub fn map(&mut self, _: u32, _: u8) {
        
        unimplemented!();
    }

    pub fn free(&mut self, _: u32) {
        unimplemented!();
    } 

    /// Opportunity to use idle time for housekeeping and reconciliation
    pub fn idle(_: &mut [u32]) {}

}


#[cfg(test)]
mod tests {

    use super::*;
    use super::entry::Range;
    use core::mem::transmute;

    #[test]
    fn test_init() {
        let mut table = Table { n_ranges: 0, ranges: [Range::null(); 511] };
        assert_eq!(table.n_ranges, 0);
        unsafe {
            let buffer = transmute::<&mut Table, *mut u32>(&mut table);
            Table::init(buffer);
        }
        assert_eq!(table.n_ranges, 1);
    }
    
}