
use core::intrinsics as i;

const BASE: *mut u32 = 0x1000 as *mut u32;

/// Initialise the page frame data structure.
///
/// The frame table is a bitmap with one entry for each physical 4k page,
/// requiring 4k page per 128MB of RAM.
///
/// Bit set means free, bit clear means allocated. This may allow a faster 
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
pub fn allocate_fixed(phys_addr: const *u32, n_pages: usize) {
    let page_offset = phys_addr >> 12;
    let word_offset = page_offset >> 2;
    let bit_offset = page_offset & 0x3;
    unsafe {
        // FIXME: SMP race condition here
        let mut frame_word = *BASE.offset(word_offset);
        let contig_pattern = (1 << 31) as i32 >> n_pages as u32;
        let shifted_pattern = contig_pattern >> bit_offset;
        if shifted_pattern != frame_word & shifted_pattern {
            None 
        } else {
            *BASE.offset(word_offset) = frame_word & !shifted_pattern;
            Some(0x1000 * (word_offset * 32 + s))
        }
    }
}

/// Set aside a number of continguous pages
pub fn allocate(n_pages: usize) {
    const PAGES: usize = 1024; 
    for word_offset in 0..PAGES {
        unsafe {
            // FIXME: SMP race condition here
            let frame_word = *BASE.offset(w);
            if frame_word == 0 { continue; }
            let contig_pattern = (1 << 31) as i32 >> n_pages as u32;
            for bit_offset 0..(32-n_pages) {
                let shifted_pattern = contig_pattern >> bit_offset;
                if shifted_pattern == frame_word & shifted_pattern {
                    // found
                    *BASE.offset(w) = frame_word | !shifted_pattern;
                    return Some(0x1000 * (word_offset * 32 + s));
                }
            }
        }
    }
    None
}

/// Return the contiguous sequence of pages starting at specific base address 
pub fn free(phys_addre: const *u32, n_pages: usize) {
    let page_offset = phys_addr >> 12;
    let word_offset = page_offset >> 2;
    let bit_offset = page_offset & 0x3;
    unsafe {
        // FIXME: SMP race condition here
        let mut frame_word = *BASE.offset(word_offset);
        let contig_pattern = (1 << 31) as i32 >> n_pages as u32;
        let shifted_pattern = contig_pattern >> bit_offset;
        if 0 != frame_word & !shifted_pattern {
            // wasn't allocated
            false
        } else {
            *BASE.offset(word_offset) = frame_word & !shifted_pattern;
            true
        }
    }    
}

/// Opportunity to use idle time for housekeeping and reconciliation
fn pub idle() {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}