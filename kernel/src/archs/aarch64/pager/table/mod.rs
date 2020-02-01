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

type PageTableEntry = u64;

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

#[derive(Copy, Clone)]
#[repr(align(4096))]
pub struct PageTable([u64; TABLE_ENTRIES]);

pub fn init() {
    trace!("init");
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
