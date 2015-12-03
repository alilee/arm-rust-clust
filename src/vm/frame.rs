//! Manages a page frame table, which holds which pages of physical memory 
//! have been allocated for virtual address ranges. 
//!
//! Pages which have not been allocated, are free memory. Since pages may 
//! be freed, there may be fragmentation, if multiple pages are required. 

/// Initialise the page frame data structure.
///
/// The frame table is a bitmap with one entry for each physical page,
/// and we don't care how big a page is.
///
/// Bit set means free, bit clear means allocated. This may allow a faster 
/// scan through fully allocated pages as they will be zeroes.
pub fn init(base: &[u32]) {
    use core::intrinsics as i;
    unsafe {
        // i::write_bytes(base, 0xFF, 4)
    }
}

/// Word containing n bits set at the top. 
fn contig_bits(n: u8) -> u32 {
    if n == 0 {
        0
    } else {
      (1u32.rotate_right(1) as i32 >> (n-1)) as u32
    }
}

/// Set aside a number of pages starting at a specific offset.
///
/// Assumes that physical memory is addressed from 0x0.
pub fn allocate_fixed(base: *mut u32, page: u32, n_pages: u8) -> Option<u32> {
    let word_offset = (page / 32) as isize;
    let bit_offset = page & 31;
    unsafe {
        // FIXME: SMP race condition here
        let frame_word = *base.offset(word_offset);
        let contig_pattern = contig_bits(n_pages);
        let shifted_pattern = contig_pattern >> bit_offset;
        if shifted_pattern == frame_word & shifted_pattern {
            *base.offset(word_offset) = frame_word & !shifted_pattern;
            Some(page)
        } else {
            None 
        }
    }
}

/// Set aside a number of continguous pages
pub fn allocate(base: *mut u32, max_pages: usize, n_pages: u8) -> Option<u32> {
    for word_offset in 0..max_pages as isize {
        unsafe {
            // FIXME: SMP race condition here
            let frame_word = *base.offset(word_offset);
            if frame_word == 0 { continue; }
            let contig_pattern = contig_bits(n_pages);
            for bit_offset in 0..(32-n_pages) {
                let shifted_pattern = contig_pattern >> bit_offset;
                if shifted_pattern == frame_word & shifted_pattern {
                    // found
                    *base.offset(word_offset) = frame_word | !shifted_pattern;
                    return Some(word_offset as u32 * 32 + (bit_offset as u32));
                }
            }
        }
    }
    None
}

/// Return the contiguous sequence of pages starting at specific base address 
pub fn free(base: *mut u32, page: u32, n_pages: u8) {
    let word_offset = (page / 32) as isize;
    let bit_offset = page & 31;
    unsafe {
        // FIXME: SMP race condition here
        let mut frame_word = *base.offset(word_offset);
        let contig_pattern = contig_bits(n_pages);
        let shifted_pattern = contig_pattern >> bit_offset;
        if 0 != frame_word & !shifted_pattern {
            panic!("Freeing unallocated frame")
        } else {
            *base.offset(word_offset) = frame_word & !shifted_pattern;
        }
    }    
}

/// Opportunity to use idle time for housekeeping and reconciliation
pub fn idle() {}

#[cfg(test)]
mod tests {

    use super::*;
    use super::contig_bits;
    
    #[test]
    fn test_contig_bits() {
        assert_eq!(contig_bits(0), 0);
        assert_eq!(contig_bits(1), 0b1 << 31);
        assert_eq!(contig_bits(2), 0b11 << 30);
        assert_eq!(contig_bits(4), 0b1111 << 28);
        assert_eq!(contig_bits(8), 0xFF << 24);
        assert_eq!(contig_bits(10), 0x3FF << 22);
    }
    
    #[test]
    fn test_init() {
        let buffer = [0u32; 4];
        init(&buffer[..]);
    }
    
}