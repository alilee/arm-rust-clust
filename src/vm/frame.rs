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
pub fn init(table: &mut [u32]) {
    for i in 0..table.len() {
        table[i] = !0;
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

/// Pattern of n bits in the correct location for a given offset.
fn shifted_pattern(n: u8, page: u32) -> u32 {
    let bit_offset = page & 31;
    let contig_pattern = contig_bits(n);
    contig_pattern >> bit_offset
}  

/// Set aside a number of pages starting at a specific offset.
///
/// Assumes that physical memory is addressed from 0x0.
pub fn allocate_fixed(table: &mut [u32], page: u32, n_pages: u8) -> Option<u32> {
    let word_offset = (page / 32) as usize;
    // FIXME: SMP race condition here
    let frame_word = table[word_offset];
    let reqd_pattern = shifted_pattern(n_pages, page);
    if reqd_pattern == frame_word & reqd_pattern {
        table[word_offset] = frame_word & !reqd_pattern;
        Some(page)
    } else {
        None 
    }
}

/// Set aside a number of continguous pages
pub fn allocate(table: &mut [u32], n_pages: u8) -> Option<u32> {
    for word_offset in 0..table.len() {
        // FIXME: SMP race condition here
        let frame_word = table[word_offset];
        if frame_word == 0 { continue; }
        for bit_offset in 0..(32-n_pages) {
            let reqd_pattern = shifted_pattern(n_pages, bit_offset as u32);
            if reqd_pattern == frame_word & reqd_pattern {
                table[word_offset] = frame_word & !reqd_pattern;
                return Some(word_offset as u32 * 32 + (bit_offset as u32));
            }
        }
    }
    None
}

/// Return the contiguous sequence of pages starting at specific table address 
pub fn free(table: &mut [u32], page: u32, n_pages: u8) {
    let word_offset = (page / 32) as usize;
    // FIXME: SMP race condition here
    let frame_word = table[word_offset];
    let reqd_pattern = shifted_pattern(n_pages, page);
    if 0 != frame_word & !reqd_pattern {
        panic!("Freeing unallocated frame")
    } else {
        table[word_offset] = frame_word | reqd_pattern;
    }
}

/// Opportunity to use idle time for housekeeping and reconciliation
pub fn idle(_: &mut [u32]) {}

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
        let mut buffer = [0xDEADBEEFu32; 4];
        init(buffer.as_mut());
        assert!(buffer.iter().all(|&x| x == 0xFFFFFFFF));
    }
    
    #[test]
    fn test_allocate_fixed() {
        let mut buffer = [0xDEADBEEFu32; 4];
        init(buffer.as_mut());
        allocate_fixed(buffer.as_mut(), 0, 1);
        assert_eq!(buffer[0], !(1 << 31));
        allocate_fixed(buffer.as_mut(), 40, 3);
        assert_eq!(buffer[1], !(0b111 << 21));
        allocate_fixed(buffer.as_mut(), 64, 1);
        allocate_fixed(buffer.as_mut(), 65, 1);
        allocate_fixed(buffer.as_mut(), 66, 1);
        assert_eq!(buffer[2], !(0b111 << 29));
        assert_eq!(None, allocate_fixed(buffer.as_mut(), 64, 1));
    }
    
    
}