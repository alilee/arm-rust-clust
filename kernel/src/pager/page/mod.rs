#![allow(dead_code)]

//! Manages mapping virtual addresses to physical addresses
//!
//! Populates the page translation tables walked by the TLB reload
//! in the ARMv7 architecture.

use core::mem::transmute;

mod l1;
mod l2;

/// The page translation tree is an l1 table and a memory-mapped array of l2 tables
///
/// Having the array means we don't have to track a stack of the l2 tables, at the
/// expense of some virtual address range. They don't have to be provisioned as long
/// as they are zeroed when they are first accessed.
pub struct Tree {
    l1_table: l1::Table,
    l2_tables: [l2::Table; 4096],
}

impl Tree {
    /// Initialise a page translation tree at a specific physical address
    pub fn init<'a>(a: *mut u32) -> &'a mut Tree {
        unsafe {
            let p_tree = transmute::<*mut u32, &mut Tree>(a);
            p_tree.l1_table.reset();
            // Test buffers must simulate zeroing of the l2 tables
            p_tree
        }
    }

    fn find_l2_table(&mut self, page: u32) -> &mut l2::Table {
        unsafe {
            let table_base = transmute::<&l2::Table, *const u32>(&self.l2_tables[0]);
            &mut self.l2_tables[self.l1_table.entry(page as usize >> 20).offset(table_base)]
        }
    }

    /// Identity-map the given number of pages starting at the base page
    /// to the same virtual addresses
    ///
    /// TODO: Use section/super-sections to save TLB entries for large contiguous mappings.
    pub fn id_map(&mut self, base_page: u32, n_pages: u8, access: u32) {
        for p in base_page..(base_page + n_pages as u32) {
            let l2_table = self.find_l2_table(p);
            l2_table.find_l2_entry(p).id_map(p, access);
        }
    }

    /*
    /// Use virtual memory mapping according to the contents of the translation table.
    ///
    /// The translation table should be populated.
    ///
    pub fn enable_mmu() {
    }

    /// Reserve a new page for an L2 translation table
    fn allocate_l2_table() -> u32 {
        0
    }

    /// Find the base address of the L2 translation table for an address
    ///
    /// Allocates a new L2 page if doesn't exist.
    fn l2_desc_base_addr(a: u32) -> u32 {
        // // find the l1 entry
        // let index = (a >> 20) as isize;
        // unsafe {
        //     let current = *page_table.offset(index);
        //     let mask_fault = 0b11;
        //     if 0 == (current & mask_fault) {
        //         // not mapped
        //         let new_page = allocate_l2_table();
        //         // point the l1 entry pointing to it
        //         *page_table.offset(index) = new_page;
        //         new_page
        //     } else {
        //         // mappped
        //         current
        //     }
        // }
        0
    }

    /// Generate an L2 translation table entry for identity mapping
    fn l2_id_desc(a: u32, page_delta: u32) -> u32 {
        // let page_size = 0x1000; // 4kb in u32
        // let page_addr = a + (page_delta * page_size); // no need to mask aligned address: & (!0xFFF);
        //
        // let mask_nG =  0b1000_0000_0000;
        // let mask_S =   0b0100_0000_0000;
        // let mask_APX = 0b0010_0000_0000;
        // let mask_TEX = 0b0001_1100_0000;
        // let mask_AP =       0b0011_0000;
        // let mask_C =             0b1000;
        // let mask_B =             0b0100;
        // let mask_small_page =    0b0010;
        // let mask_XN =            0b0001;
        //
        // let entry_flags = mask_small_page;
        //
        // page_addr | entry_flags
        0
    }
 
    /// Identity map the pages at the given page-aligned address into the translation table at the same address
    ///
    /// All pages must be in the same 1MB-aligned section.
    pub fn id_map(a: u32, pages: u32) -> u32 {
        // let l2_base = l2_desc_base_addr(a);
        // let mask_page_index = 0x8F;
        // let first_page_index = (a >> 12) & mask_page_index;
        //
        // for p in 0..pages {
        //     let current = l2_id_desc(a, p);
        //     let l2_base_ptr = l2_base as *mut u32;
        //     let page_index = (first_page_index + p) as isize;
        //     unsafe {
        //         *l2_base_ptr.offset(page_index) = current;
        //     }
        // }
        // pages
        0
    }
*/
}


#[cfg(test)]
mod tests {

    use super::*;
    use super::l1;
    use super::l2;
    use core::mem::transmute;

    #[test]
    fn test_init() {
        // let sandwich = [ [ 0u32; 1024 ]; 4096];

        // let mut tree = Tree {   l1_table: l1::Table { entries: [ l1::Entry::init_fault(0xDEADBEEF); 1024] },
        //                         l2_tables: [ l2::Table { entries: [ l2::Entry::init(0xDEADBEEF); 1024] }; 4096] };
        // { l1_table: l1::Table::init(0xDEADBEEF),
        //                      l2_tables: [ l2::Table::init(0xDEADBEEF); 4096 ] };
        // assert!(!tree.l1_table.entries[0].is_fault());
        // assert!(!tree.l2_tables[0].entries[0].is_fault());
        // unsafe {
        //     let buffer = transmute::<&mut Tree, *mut u32>(&mut tree);
        //     Tree::init(buffer);
        // }
        // assert!(tree.l1_table.entries[0].is_fault());
        // for l2_table in tree.l2_tables.iter() {
        //     for e in l2_table.entries.iter() {
        //         assert!(e.is_fault());
        //     }
        // }
    }

}
