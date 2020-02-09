pub use super::attrs::TranslationAttributes;
use super::desc;
use super::mair;
pub use super::trans::Translation;

use crate::pager::{MemOffset, PAGESIZE_BYTES};
use crate::util::{set_above_bits, set_below_bits};

#[allow(unused_imports)]
use log::{debug, info, trace};

use core::mem;
use core::slice::Iter;

pub type PageTableEntryType = u64;

#[derive(Copy, Clone)]
pub struct PageTableEntry(PageTableEntryType);

const TABLE_ENTRIES: usize = PAGESIZE_BYTES / mem::size_of::<PageTableEntry>();
const LOWER_VA_BITS: u32 = 48; // 256 TB
const UPPER_VA_BITS: u32 = 39; // 512 GB
pub const LOWER_VA_TOP: usize = 1 << (LOWER_VA_BITS as usize);
pub const UPPER_VA_BASE: usize = set_above_bits(UPPER_VA_BITS);
pub const UPPER_TABLE_LEVEL: u8 = 1;
pub const LOWER_TABLE_LEVEL: u8 = 0;
pub const LEVEL_OFFSETS: [usize; 4] = [39, 30, 21, 12];
pub const LEVEL_WIDTH: usize = 9;

//pub const fn kernel_va_bits() -> u32 {
//    UPPER_VA_BITS
//}
//
//pub const fn user_va_bits() -> u32 {
//    LOWER_VA_BITS
//}

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

pub fn init() -> Result<(), u64> {
    use cortex_a::{
        barrier,
        regs::{TCR_EL1::*, *},
    };

    trace!("init");
    info!("kernel base: {:#x}", UPPER_VA_BASE);
    trace!(
        "kernel va space: {:#x}",
        0xFFFF_FFFF_FFFF_FFFF - UPPER_VA_BASE
    );
    mair::init();

    let mem_offset = MemOffset::identity();
    let tt1 = Translation::new_upper(mem_offset)?;
    let tt0 = Translation::new_lower(mem_offset)?;

    let ttbr1 = tt1.base_register();
    let ttbr0 = tt0.base_register();

    let asid = 0;
    let ttbr0: u64 = ttbr0 | ((asid as u64) << 48);

    TTBR0_EL1.set(ttbr0);
    TTBR1_EL1.set(ttbr1);

    assert_eq!(crate::pager::PAGESIZE_BYTES, 4096);
    //
    // TODO: nTWE, nTWI
    //
    TCR_EL1.modify(
        AS::Bits_16    // 16 bit ASID 
            + IPS::Bits_36  // 36 bits/64GB of physical address space
            + TG1::KiB_4
            + SH1::Outer
            + ORGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN1::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T1SZ.val(64 - UPPER_VA_BITS as u64) // 64-t1sz=43 bits of address space in high range
            + TG0::KiB_4
            + SH0::Outer
            + ORGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + IRGN0::WriteThrough_ReadAlloc_NoWriteAlloc_Cacheable
            + T0SZ.val(64 - LOWER_VA_BITS as u64), // 64-t0sz=48 bits of address space in low range
    );
    unsafe {
        barrier::isb(barrier::SY);
    }

    Ok(())
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
