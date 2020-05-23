// SPDX-License-Identifier: Unlicense

//! Page table data structures.

use crate::pager::PAGESIZE_BYTES;
use crate::util::bitfield::{register_bitfields, Bitfield};

use core::mem;
use core::ops::{Index, IndexMut};

pub type PageTableEntryType = u64;

/// An entry in one of the levels of an ARMv8 page table.
#[derive(Copy, Clone)]
pub struct PageTableEntry(PageTableEntryType);

impl PageTableEntry {
    pub const fn is_valid(self) -> bool {
        0 != self.0 & 1
    }
}

/// Number of entries in a page table.
pub const TABLE_ENTRIES: usize = PAGESIZE_BYTES / mem::size_of::<PageTableEntry>();
/// Bit positions of the table offsets within a virtual address.
pub const LEVEL_OFFSETS: [usize; 4] = [39, 30, 21, 12];
/// Width of the table offsets within a virtual address.
pub const LEVEL_WIDTH: usize = 9;

#[derive(Copy, Clone)]
#[repr(align(4096))]
pub struct PageTable([PageTableEntry; TABLE_ENTRIES]);

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

register_bitfields! {
    u64,
    pub TableDescriptorFields [
        NSTable OFFSET(63) NUMBITS(1) [],
        APTable OFFSET(61) NUMBITS(2) [
            NoEffect = 0b00,
            PrivOnly = 0b01,
            ReadOnly = 0b10,
            PrivReadOnly = 0b11
        ],
        UXNTable OFFSET(60) NUMBITS(1) [],
        PXNTable OFFSET(59) NUMBITS(1) [],
        NextLevelTableAddress OFFSET(12) NUMBITS(35) [],
        Type OFFSET(1) NUMBITS(1) [
            Reserved = 0b0,
            Page = 0b1
        ],
        Valid OFFSET(0) NUMBITS(1) []
    ]
}

pub type TableDescriptor = Bitfield<PageTableEntryType, TableDescriptorFields::Register>;

impl TableDescriptor {}

impl From<PageTableEntry> for TableDescriptor {
    fn from(pte: PageTableEntry) -> Self {
        Self::new(pte.0)
    }
}

register_bitfields! {
    u64,
    pub PageBlockDescriptorFields [
        Available OFFSET(55) NUMBITS(9) [],
        UXN OFFSET(54) NUMBITS(1) [],                      // Unprivileged Execute Never
        PXN OFFSET(53) NUMBITS(1) [],                      // Privileged Execute Never
        Contiguous OFFSET(52) NUMBITS(1) [],               // One of a contiguous set of entries
        Dirty OFFSET(51) NUMBITS(1) [],                    // Dirty (DBM?)
        OutputAddress OFFSET(12) NUMBITS(35) [],
        nG OFFSET(11) NUMBITS(1) [],                       // Not Global - all or current ASID
        AF OFFSET(10) NUMBITS(1) [],                       // Access flag
        SH OFFSET(8) NUMBITS(2) [                          // Shareability
            NonShareable = 0b00,
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],
        AP OFFSET(6) NUMBITS(2) [                          // Data access permissions
            PrivOnly = 0b00,
            ReadWrite = 0b01,
            PrivReadOnly = 0b10,
            ReadOnly = 0b11
        ],
        AttrIndx OFFSET(2) NUMBITS(3) [],                  // Memory attributes index for MAIR_ELx
        Type OFFSET(1) NUMBITS(1) [
            Block = 0b0,
            Table = 0b1
        ],
        Valid OFFSET(0) NUMBITS(1) []
    ]
}

pub type PageBlockDescriptor = Bitfield<PageTableEntryType, PageBlockDescriptorFields::Register>;

impl PageBlockDescriptor {}

impl From<PageTableEntry> for PageBlockDescriptor {
    fn from(pte: PageTableEntry) -> Self {
        Self::new(pte.0)
    }
}
