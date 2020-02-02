pub mod attrs;
pub mod desc;
mod mair;
mod trans;

use super::virt_addr::VirtOffset;
pub use attrs::TranslationAttributes;
pub use trans::Translation;

use crate::pager::PAGESIZE_BYTES;
use crate::util::set_above_bits;

#[allow(unused_imports)]
use log::{debug, info, trace};

use core::mem;
use core::slice::Iter;

type PageTableEntryType = u64;
#[derive(Copy, Clone)]
pub struct PageTableEntry(PageTableEntryType);

const TABLE_ENTRIES: usize = PAGESIZE_BYTES / mem::size_of::<PageTableEntry>();
const LOWER_VA_BITS: u32 = 48; // 256 TB
const UPPER_VA_BITS: u32 = 39; // 512 GB
pub const UPPER_VA_BASE: usize = set_above_bits(UPPER_VA_BITS);
const UPPER_TABLE_LEVEL: u8 = 1;
const LOWER_TABLE_LEVEL: u8 = 0;
const LEVEL_OFFSETS: [usize; 4] = [39, 30, 21, 12];
const LEVEL_WIDTH: usize = 9;

pub const fn kernel_mem_offset() -> VirtOffset {
    VirtOffset::new_const(UPPER_VA_BASE)
}

pub const fn kernel_va_bits() -> u32 {
    UPPER_VA_BITS
}

pub const fn user_va_bits() -> u32 {
    LOWER_VA_BITS
}

impl PageTableEntry {
    pub fn get(&self) -> PageTableEntryType {
        self.0
    }
    pub fn is_valid(&self) -> bool {
        0 != self.0 & 0b1
    }
    pub fn is_table(&self, level: usize) -> bool {
        assert!(self.is_valid());
        level != 3 && (0 != self.0 & 0b10)
    }
}

impl From<desc::PageBlockDescriptor> for PageTableEntry {
    fn from(page_desc: desc::PageBlockDescriptor) -> Self {
        Self(page_desc.get())
    }
}

impl From<desc::TableDescriptor> for PageTableEntry {
    fn from(table_desc: desc::TableDescriptor) -> Self {
        Self(table_desc.get())
    }
}

#[derive(Copy, Clone)]
#[repr(align(4096))]
pub struct PageTable([PageTableEntry; TABLE_ENTRIES]);

impl PageTable {
    pub fn entries(&self) -> Iter<PageTableEntry> {
        self.0.iter()
    }
    pub fn get(&self, index: usize) -> PageTableEntry {
        self.0[index]
    }
    pub fn set(&mut self, index: usize, pte: PageTableEntry) {
        self.0[index] = pte;
    }
}

pub fn init() {
    trace!("init");
    info!("kernel base: {:#x}", UPPER_VA_BASE);
    trace!(
        "kernel va space: {:#x}",
        0xFFFF_FFFF_FFFF_FFFF - UPPER_VA_BASE
    );

    mair::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consts() {
        assert_eq!(TABLE_ENTRIES, 512);
        assert_eq!(LOWER_VA_BITS, 48);
        assert_eq!(UPPER_VA_BITS, 43);
        assert_eq!(UPPER_VA_BASE, 0xFFFF_FE00_0000_0000);
        assert_eq!(UPPER_TABLE_LEVEL, 1);
        assert_eq!(LOWER_TABLE_LEVEL, 0);
    }
}
