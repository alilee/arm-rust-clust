//! Manages a page frame table, which holds which pages of physical memory 
//! have been allocated for virtual address ranges 
//!
//! Pages which have not been allocated, are free memory. Since pages may 
//! be freed, there may be fragmentation, if multiple pages are required. 

use core::mem::transmute;

/// The frame table is the highest currently used page and a stack of the free pages
/// residing under that mark
pub struct Table {
    free_page_nos: [u32; 1024-2],
    highwater_mark: u32,    // page number of highest page previously allocated. 
    n_free: usize,          // offset to empty position above the top of stack (ie. 0 when empty)  
}

#[allow(dead_code)]
impl Table {
    
    /// Initialise the page frame data structure into a specific physical address
    pub fn init<'a>(a: *mut u32) -> &'a mut Table {
        unsafe {
            let p_table = transmute::<*mut u32, &mut Table>(a); 
            p_table.n_free = 0;
            p_table.highwater_mark = 0;
            p_table
        }
    }
    
    fn raise_hwm(&mut self, page: u32, n_pages: u8) -> Option<u32> {
        // TODO: find sequences not above current high-water mark
        // TODO: out of range if (page + n_pages) > physical memory.
        if (page + n_pages as u32) < self.highwater_mark {
            None
        } else {
            // TODO: out of free list space if stack_top + (page - self.highwater_mark) > stack_entries.length()
            for p in 0..(page - self.highwater_mark) {
                self.free_page_nos[self.n_free + p as usize] = self.highwater_mark + p;
            }
            self.n_free += page as usize - self.highwater_mark as usize;
            self.highwater_mark = page + n_pages as u32;
            Some(page)
        }        
    }

    /// Set aside a number of pages starting at a specific offset
    pub fn allocate_fixed(&mut self, page: u32, n_pages: u8) -> Option<u32> {
        match self.raise_hwm(page, n_pages) {
            None => None,
            Some(_) => Some(page),
        }
    }

    /// Set aside a number of contiguous pages
    pub fn allocate(&mut self, n_pages: u8) -> Option<u32> {
        // TODO: grab contiguous pages from freelist (just top?) 
        if n_pages == 1 && self.n_free > 0 {
            self.n_free -= 1;
            Some(self.free_page_nos[self.n_free])
        } else {
            let hwm = self.highwater_mark;
            self.allocate_fixed(hwm, n_pages)
        }
    }

    /// Return the contiguous sequence of pages starting at specific table address 
    pub fn free(&mut self, page: u32, n_pages: u8) {
        for p in 0..n_pages {
            self.free_page_nos[self.n_free + p as usize] = page + p as u32;
        }
        self.n_free += n_pages as usize;
    }

    /// Opportunity to use idle time for housekeeping and reconciliation
    pub fn idle(&self) {
        // TODO: detect duplicates and panic
        // TODO: detect page_nos above high-water mark and panic
        // TODO: sort the free page_nos and retract high-water mark
    }

}

#[cfg(test)]
mod tests {

    use super::*;
    use core::mem::transmute;
    
    #[test]
    fn test_init() {
        let mut table = Table { highwater_mark: 1, n_free: 1, free_page_nos: [99; 1024-2] };
        assert_eq!(table.highwater_mark, 1);
        assert_eq!(table.n_free, 1);
        unsafe {
            let buffer = transmute::<&mut Table, *mut u32>(&mut table);
            Table::init(buffer);
        }
        assert_eq!(table.highwater_mark, 0);
        assert_eq!(table.n_free, 0);
    }
    
    #[test]
    fn test_allocation() {
        let mut buffer = Table { highwater_mark: 1, n_free: 1, free_page_nos: [99; 1024-2] };
        let table = unsafe {
                            let p_buffer = transmute::<&mut Table, *mut u32>(&mut buffer);
                            Table::init(p_buffer)
                        };
        assert_eq!(table.allocate(1), Some(0));
        assert_eq!(table.highwater_mark, 1);
        assert_eq!(table.n_free, 0);
        assert_eq!(table.allocate_fixed(40, 3), Some(40));
        assert_eq!(table.highwater_mark, 43);
        assert_eq!(table.n_free, 39);
        assert_eq!(table.free_page_nos[0], 1);
        assert_eq!(table.free_page_nos[38], 39);
        assert_eq!(table.allocate(1), Some(39));
        assert_eq!(table.highwater_mark, 43);
        assert_eq!(table.n_free, 38);
        assert_eq!(table.free_page_nos[0], 1);
        assert_eq!(table.free_page_nos[37], 38);
        assert_eq!(table.allocate(3), Some(43));
        assert_eq!(table.highwater_mark, 46);
        assert_eq!(table.n_free, 38);
        assert_eq!(table.free_page_nos[0], 1);
        assert_eq!(table.free_page_nos[37], 38);
        assert_eq!(table.allocate_fixed(64, 1), Some(64));
        assert_eq!(table.allocate_fixed(65, 1), Some(65));
        assert_eq!(table.allocate_fixed(66, 1), Some(66));
        assert_eq!(table.highwater_mark, 67);
        assert_eq!(table.n_free, 56);
        assert_eq!(table.free_page_nos[0], 1);
        assert_eq!(table.free_page_nos[55], 63);
    }
    
    #[test]
    fn test_free() {
        let mut buffer = Table { highwater_mark: 1, n_free: 1, free_page_nos: [99; 1024-2] };
        let table = unsafe {
                            let p_buffer = transmute::<&mut Table, *mut u32>(&mut buffer);
                            Table::init(p_buffer)
                        };
        assert_eq!(table.allocate(1), Some(0));
        assert_eq!(table.allocate_fixed(40, 3), Some(40));
        table.free(0, 1);
        assert_eq!(table.highwater_mark, 43);
        assert_eq!(table.n_free, 40);
        assert_eq!(table.free_page_nos[0], 1);
        assert_eq!(table.free_page_nos[39], 0);
    }
    
    
}