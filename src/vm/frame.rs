
use core::intrinsics as i;

const BASE: *mut u32 = 0x1000 as *mut u32;

/// Initialise the page frame data structure.
///
/// The frame table is a bitmap with one entry for each physical 4k page,
/// requiring 4k page per 128MB of RAM.
///
/// Bit set means free, bit clear means allocated. This is to allow a fast 
/// scan through fully allocated pages as they will be zeroes.
pub fn init() {
    unsafe {
        i::write_bytes(BASE, -1, 0x1000)
    }
    BASE
}

/// Set aside a number of pages starting at a specific physical address.
///
/// Assumes that physical memory is addressed from 0x0.
pub fn allocate_fixed(phys_addr: *const u32, n_pages: usize) {
    for w in 0..1024 {
        unsafe {
            let word = *BASE.offset(w);
            if word == 0 { continue; }
            const SEQ: u32 = (0x10000000 as i32 >> n_pages);
            for s in 0..(32-n_pages) {
                if SEQ == word & SEQ {
                    // found
                    // FIXME: race condition here
                    *BASE.offset(w) = word | SEQ;
                    return 
                }
            }
        }
    }
    
}

/// Set aside a number of continguous pages
pub fn allocate(n_pages: usize) {
    for w in 0..1024 {
        unsafe {
            let word = *BASE.offset(w);
            if word == 0 { continue; }
            const SEQ: u32 = (0x10000000 as i32 >> n_pages);
            for s in 0..(32-n_pages) {
                if SEQ == word & SEQ {
                    // found
                    // FIXME: race condition here
                    *BASE.offset(w) = word | SEQ;
                    return 
                }
            }
        }
    }
}

/// Return the contiguous sequence of pages starting at specific base address 
pub fn free(a: *cost *u32, n_pages: usize) {
    
}

/// Opportunity to use idle time for housekeeping and reconciliation
fn pub idle() {}