//! Manages a page frame table, which tracks which pages of physical memory
//! have been allocated for virtual address ranges
//!
//! Pages which have not been allocated, are free memory. Since pages may
//! be freed, there may be fragmentation, if multiple pages are required.

use super::{PhysAddr, PhysAddrRange};

use log::{debug, info};

const MAX_MEMORY: usize = (258 * 1024 * 1024);
const PAGE_SIZE: usize = 4096;
const MAX_FRAMES: usize = MAX_MEMORY / PAGE_SIZE;

/// The frame map records which physical pages have been reserved and which are available.
struct FrameMap {
    page_map: [u64; MAX_FRAMES / 64], // bitmap of pages where 0 is available and 1 is allocated
    highwater_mark: usize,            // index of lowest page map entry that has any free
    range: PhysAddrRange,             // range of physical memory covered by this map
}

#[allow(dead_code)]
impl FrameMap {
    pub const fn init() -> Self {
        FrameMap {
            page_map: [0; MAX_FRAMES / 64],
            highwater_mark: 0,
            range: PhysAddrRange {
                base: PhysAddr(0),
                length: 0,
            },
        }
    }

    /// Initialise the page frame data structure into a specific physical address
    pub fn reset(self: &mut Self, range: PhysAddrRange) -> Result<(), u64> {
        for x in self.page_map.iter_mut() {
            *x = 0
        }
        self.range = range;
        self.highwater_mark = 0;
        Ok(())
    }

    pub fn reserve(self: &mut Self, range: PhysAddrRange) -> Result<(), u64> {
        if range.outside(&self.range) {
            return Err(0);
        }
        info!("reserving: {:?}", range);

        let first_page = PhysAddrRange::bounded_by(self.range.base, range.base).pages(PAGE_SIZE);
        let mut i = first_page / 64;
        let offset = first_page % 64;
        let mut n_pages_reqd = range.pages(PAGE_SIZE);
        while n_pages_reqd > 0 {
            let chunk = self.page_map[i];
            let pages = core::cmp::min(n_pages_reqd, 64 - offset);
            let reserve = ((1u64 << (pages as u64 + 1u64)) - 1u64) << (offset as u64);
            if chunk & reserve > 0 {
                return Err(0);
            }
            self.page_map[i] = chunk | reserve;
            n_pages_reqd -= pages;
            i += 1;
        }
        if first_page == self.highwater_mark {
            let last_chunk = self.range.length / PAGE_SIZE;
            while self.highwater_mark < last_chunk
                && self.page_map[self.highwater_mark].count_ones() == 64
            {
                self.highwater_mark += 1;
            }
        }
        Ok(())
    }

    pub fn print_state(self: &mut Self) {
        debug!("Frame map covering: {:?}", self.range);

        let mut gap_i = 0usize;
        let mut last_printed_addr = 0u64;
        for (i, chunk) in self.page_map.iter().enumerate() {
            let addr = self.range.base.offset(i as u64 * PAGE_SIZE as u64).0;
            if i == self.highwater_mark {
                debug!("0x{:08x} ====================== highwater", addr);
                last_printed_addr = addr;
            }
            if *chunk > 0 {
                if gap_i > 0 {
                    debug!("... 0x{:08x}", addr - last_printed_addr);
                    gap_i = 0;
                }
                //                let mut chunk_image = ['.' as u8; 64];
                //                let mut bits = *chunk;
                //                for i in 0..64 {
                //                    if bits & 1 != 0 {
                //                        chunk_image[i] = 'X' as u8;
                //                    }
                //                    bits = bits >> 1;
                //                }
                //                let s = unsafe { core::str::from_utf8_unchecked(&chunk_image) };
                debug!("0x{:08x}: {:064b}", addr, chunk.reverse_bits());
                last_printed_addr = addr;
            } else {
                gap_i += 1;
            }
        }
        let addr = self.range.top().0;
        debug!("... 0x{:08x}", addr - last_printed_addr);
    }
}

static mut FRAME_MAP: FrameMap = FrameMap::init();

fn get_frame_map() -> &'static mut FrameMap {
    unsafe { &mut FRAME_MAP }
}

pub fn reset(range: PhysAddrRange) {
    get_frame_map().reset(range);
}

pub fn reserve(range: PhysAddrRange) -> Result<(), u64> {
    get_frame_map().reserve(range)
}

pub fn print_state() {
    get_frame_map().print_state();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let ft = FrameMap::init();
        let ram = PhysAddrRange(0x40_000_000, 0x10_000_000);
        ft.reset(ram);
        ft.print_state();
        let range = PhysAddrRange(0x40_000_000, 0x1);
        ft.reserve(range);
        ft.print_state();
        assert!(true);
    }
}
