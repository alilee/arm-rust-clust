// SPDX-License-Identifier: Unlicense

//! Page table data structures.

use crate::pager::{AttributeField, Attributes, PhysAddr, PAGESIZE_BYTES};
use crate::util::bitfield::{register_bitfields, Bitfield, FieldValue};

use core::mem;
use core::ops::{Index, IndexMut};
use core::fmt::{Debug, Formatter};

pub type PageTableEntryType = u64;

/// An entry in one of the levels of an ARMv8 page table.
#[derive(Copy, Clone)]
pub struct PageTableEntry(PageTableEntryType);

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "PageTableEntry({:#x})", self.0)
    }
}

impl PageTableEntry {
    pub const fn is_valid(self) -> bool {
        0 != self.0 & 1
    }

    pub const fn next_level_table_address(self) -> PhysAddr {
        const MASK: usize = ((1 << (48 - 12)) - 1) << 12;
        PhysAddr::at((self.0 as usize) & MASK)
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
type TableDescriptorMask = FieldValue<PageTableEntryType, TableDescriptorFields::Register>;

impl From<Attributes> for TableDescriptorMask {
    fn from(attributes: Attributes) -> Self {
        use AttributeField::*;
        use TableDescriptorFields::*;

        const ATTRIBUTE_FIELD_MAP: &[(bool, AttributeField, TableDescriptorMask)] = &[
            (false, UserExec, UXNTable::SET),
            (false, KernelExec, PXNTable::SET),
        ];

        let mut result = Self::new(0, 0, 0);
        for (agree, attribute, field) in ATTRIBUTE_FIELD_MAP {
            if *agree == (attributes.is_set(*attribute)) {
                result += *field;
            }
        }

        result
    }
}

impl TableDescriptor {
    /// Create a new table descriptor entry from attributes.
    pub fn new_entry(phys_addr: PhysAddr, attributes: Attributes) -> Self {
        use TableDescriptorFields::*;

        let mut field = TableDescriptorMask::from(attributes);
        field += Valid::SET
            + Type::Page
            + NextLevelTableAddress.val(phys_addr.page() as PageTableEntryType);
        Self::from(field)
    }
}

impl From<TableDescriptor> for PageTableEntry {
    fn from(desc: TableDescriptor) -> Self {
        Self(desc.get())
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
type PageBlockDescriptorMask = FieldValue<PageTableEntryType, PageBlockDescriptorFields::Register>;

impl From<Attributes> for PageBlockDescriptorMask {
    fn from(attributes: Attributes) -> Self {
        use AttributeField::*;
        use PageBlockDescriptorFields::*;
        use super::mair::MAIR;

        const ATTRIBUTE_FIELD_MAP: &[(bool, AttributeField, PageBlockDescriptorMask)] = &[
            (false, UserExec, UXN::SET),
            (false, KernelExec, PXN::SET),
            (true, OnDemand, AF::SET),
        ];

        let mut result = Self::from(SH::OuterShareable);
        for (agree, attribute, field) in ATTRIBUTE_FIELD_MAP {
            if *agree == (attributes.is_set(*attribute)) {
                result += *field;
            }
        }

        match (
            attributes.is_set(UserRead),
            attributes.is_set(UserWrite),
            attributes.is_set(KernelRead),
            attributes.is_set(KernelWrite),
        ) {
            (true, true, true, true) => {
                result += AP::ReadWrite;
            }
            (false, false, true, true) => {
                result += AP::PrivOnly;
            }
            (true, false, true, false) => {
                result += AP::ReadOnly;
            }
            (false, false, true, false) => {
                result += AP::PrivReadOnly;
            }
            (false, false, false, false) => { // presumably execute-only
                result += AP::PrivReadOnly;
            }
            _ => panic!(),
        }

        if attributes.is_set(Device) {
            result += AttrIndx.val(MAIR::DeviceStronglyOrdered as u64);
        } else {
            result += AttrIndx.val(MAIR::MemoryWriteThrough as u64);
        }

        result
    }
}

impl PageBlockDescriptor {
    /// Create a new page block descriptor entry from attributes.
    pub fn new_entry(
        level: u8,
        output_addr: PhysAddr,
        attributes: Attributes,
        contiguous: bool,
    ) -> Self {
        use AttributeField::*;
        use PageBlockDescriptorFields::*;

        let mut field = PageBlockDescriptorMask::from(attributes);
        if !attributes.is_set(OnDemand) {
            field += Valid::SET + OutputAddress.val(output_addr.page() as PageTableEntryType);
        }
        if contiguous {
            field += Contiguous::SET;
        }
        if level == 3 {
            field += Type::Block;
        }
        Self::from(field)
    }
}

impl From<PageBlockDescriptor> for PageTableEntry {
    fn from(desc: PageBlockDescriptor) -> Self {
        Self(desc.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_descriptor() {
        let result = TableDescriptor::new(99);
        assert_eq!(99, result.get());
    }

    #[test]
    fn test_table_descriptor_entry() {
        let next_level_table_addr = PhysAddr::at(0x123_0000);
        let attributes = Attributes::DEVICE;
        let result = TableDescriptor::new_entry(next_level_table_addr, attributes);
        trace!("{:x}", result.get());
        assert_eq!(0x1800000001230003, result.get());
    }

    #[test]
    fn test_table_descriptor_mask() {
        use crate::pager::Attributes;
        let mut field = TableDescriptorMask::from(Attributes::DEVICE);

        use TableDescriptorFields::*;
        field += Valid::SET;

        let result = TableDescriptor::from(field);
        trace!("{:x}", result.get());
        assert_eq!(0x1800000000000001, result.get())
    }

    #[test]
    fn test_page_block_descriptor() {
        let result = PageBlockDescriptor::new(99);
        assert_eq!(99, result.get());
    }

    #[test]
    fn test_page_block_descriptor_entry() {
        let level = 3;
        let output_addr = PhysAddr::at(0x123_0000);
        let attributes = Attributes::DEVICE;
        let contiguous = true;
        let result = PageBlockDescriptor::new_entry(level, output_addr, attributes, contiguous);
        trace!("{:x}", result.get());
        assert_eq!(0x70000001230201, result.get());
    }

    #[test]
    fn test_page_block_descriptor_mask() {
        use crate::pager::Attributes;
        let mut field = PageBlockDescriptorMask::from(Attributes::DEVICE);

        use PageBlockDescriptorFields::*;
        field += Contiguous::SET + Dirty::SET + nG::SET + Valid::SET;

        let result = PageBlockDescriptor::from(field);
        trace!("{:x}", result.get());
        assert_eq!(0x78000000000a01, result.get())
    }
}
