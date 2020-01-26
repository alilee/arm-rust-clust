//! Manages a page frame table, which tracks which pages of physical memory
//! have been allocated for virtual address ranges
//!
//! Pages which have not been allocated, are free memory. Since pages may
//! be freed, there may be fragmentation, if multiple pages are required.

use super::PAGESIZE_BYTES;
use super::{PhysAddr, PhysAddrRange};

use crate::util::{locked::Locked, set_below_bits};
use log::{debug, info};

use core::mem;

const MAX_MEMORY: usize = (256 * 1024 * 1024);
const MAX_FRAMES: usize = MAX_MEMORY / PAGESIZE_BYTES;
const MAP_ENTRIES: usize = MAX_FRAMES / 64;

/// The frame map records which physical pages have been reserved and which are available.
pub struct FrameMap {
    page_map: [usize; MAP_ENTRIES],
    // bitmap of pages where 0 is available and 1 is allocated
    highwater_mark: usize,
    // index of lowest page map entry that has any free
    range: PhysAddrRange, // range of physical memory covered by this map
}

#[allow(dead_code)]
impl FrameMap {
    const CHUNK_SIZE: usize = mem::size_of::<usize>();
    const CHUNK_BITS: usize = mem::size_of::<usize>() * 8;

    pub const fn init() -> Self {
        Self {
            page_map: [0; MAP_ENTRIES],
            highwater_mark: 0,
            range: PhysAddrRange::new_const(PhysAddr::new_const(0), 0),
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

    fn raise_highwater_mark(&mut self) {
        let last_chunk = self.range.length() / (PAGESIZE_BYTES * Self::CHUNK_BITS);
        while self.highwater_mark < last_chunk && self.page_map[self.highwater_mark] == !0 {
            self.highwater_mark += 1;
        }
    }

    pub fn reserve(self: &mut Self, range: PhysAddrRange) -> Result<(), u64> {
        if range.outside(&self.range) {
            return Err(0);
        }
        info!("reserving: {:?}", range);

        let first_page = PhysAddrRange::bounded_by(self.range.base(), range.base()).pages();
        let mut i = first_page / Self::CHUNK_BITS;
        let offset = first_page % Self::CHUNK_BITS;
        let mut n_pages_reqd = range.pages();
        while n_pages_reqd > 0 {
            let chunk = self.page_map[i];
            let pages = core::cmp::min(n_pages_reqd, 64 - offset);
            let reserve = ((1 << (pages + 1)) - 1) << offset;
            if (chunk & reserve) > 0 {
                return Err(0);
            }
            self.page_map[i] = chunk | reserve;
            n_pages_reqd -= pages;
            i += 1;
        }
        self.raise_highwater_mark();
        Ok(())
    }

    pub fn find_contiguous(&mut self, n_pages: u32) -> Result<PhysAddrRange, u64> {
        for i in self.highwater_mark..MAP_ENTRIES {
            let chunk = self.page_map[i];
            if chunk.count_zeros() > n_pages {
                let mut page_bits = set_below_bits(n_pages);
                for bit in 0..(Self::CHUNK_BITS - (n_pages as usize)) {
                    if chunk & page_bits == 0 {
                        // available
                        let base_page = (i * Self::CHUNK_BITS) + bit;
                        let offset = base_page * PAGESIZE_BYTES;
                        let result = PhysAddrRange::new(
                            self.range.base().offset(offset),
                            n_pages as usize * PAGESIZE_BYTES,
                        );
                        self.page_map[i] |= page_bits;
                        self.raise_highwater_mark();
                        return Ok(result);
                    }
                    page_bits <<= 1;
                }
            }
        }
        self.raise_highwater_mark();
        Err(0)
    }

    pub fn print_state(self: &mut Self) {
        debug!("Frame map covering: {:?}", self.range);

        let mut gap_i = 0usize;
        let mut last_printed_addr = 0usize;
        for (i, chunk) in self.page_map.iter().enumerate() {
            let addr = self.range.base().offset(i * PAGESIZE_BYTES).get();
            if i == self.highwater_mark {
                debug!("0x{:08x} ====================== highwater", addr);
                last_printed_addr = addr;
            }
            if *chunk > 0 {
                if gap_i > 0 {
                    debug!("... 0x{:08x}", addr - last_printed_addr);
                    gap_i = 0;
                }
                debug!("0x{:08x}: {:064b}", addr, chunk.reverse_bits());
                last_printed_addr = addr;
            } else {
                gap_i += 1;
            }
        }
        let addr = self.range.top().get();
        debug!("... 0x{:08x}", addr - last_printed_addr);
    }
}

static FRAME_MAP: Locked<FrameMap> = Locked::<FrameMap>::new(FrameMap::init());

pub fn reset(map_range: PhysAddrRange) -> Result<(), u64> {
    FRAME_MAP.lock().reset(map_range)
}

pub fn reserve(range: PhysAddrRange) -> Result<(), u64> {
    FRAME_MAP.lock().reserve(range)
}

pub fn find() -> Result<PhysAddr, u64> {
    let par = FRAME_MAP.lock().find_contiguous(1)?;
    Ok(par.base())
}

pub fn print_state() {
    FRAME_MAP.lock().print_state();
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
