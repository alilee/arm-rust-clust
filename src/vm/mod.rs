
mod frame;

use core::intrinsics as i;

extern {
    static page_table: *mut u32;
}

/// Empty the page table
pub fn init() {
    // Zero four pages to clear the L1 translation table
    unsafe {
        i::write_bytes(page_table, 0, 0x1000);
    }
}

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
    // find the l1 entry
    let index = (a >> 20) as isize;
    unsafe {
        let current = *page_table.offset(index);
        let mask_fault = 0b11;
        if 0 == (current & mask_fault) {
            // not mapped
            let new_page = allocate_l2_table();
            // point the l1 entry pointing to it
            *page_table.offset(index) = new_page;
            new_page
        } else {
            // mappped
            current
        }
    }
}

/// Generate an L2 translation table entry for identity mapping
fn l2_id_desc(a: u32, page_delta: u32) -> u32 {
    let page_size = 0x1000; // 4kb in u32
    let page_addr = a + (page_delta * page_size); // no need to mask aligned address: & (!0xFFF);

    let mask_nG =  0b1000_0000_0000;
    let mask_S =   0b0100_0000_0000;
    let mask_APX = 0b0010_0000_0000;
    let mask_TEX = 0b0001_1100_0000;
    let mask_AP =       0b0011_0000;
    let mask_C =             0b1000;
    let mask_B =             0b0100;
    let mask_small_page =    0b0010;
    let mask_XN =            0b0001;

    let entry_flags = mask_small_page;

    page_addr | entry_flags
}
 
/// Identity map the pages at the given page-aligned address into the translation table at the same address
///
/// All pages must be in the same 1MB-aligned section.
pub fn id_map(a: u32, pages: u32) -> u32 {
    let l2_base = l2_desc_base_addr(a);
    let mask_page_index = 0x8F;
    let first_page_index = (a >> 12) & mask_page_index;
    
    for p in 0..pages {
        let current = l2_id_desc(a, p);
        let l2_base_ptr = l2_base as *mut u32;
        let page_index = (first_page_index + p) as isize; 
        unsafe {
            *l2_base_ptr.offset(page_index) = current;
        }
    }
    pages
}
