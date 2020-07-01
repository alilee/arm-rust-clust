// SPDX-License-Identifier: Unlicense

//! Page table data structures.

use crate::pager::{Addr, AttributeField, Attributes, PhysAddr, PAGESIZE_BYTES};
use crate::util::bitfield::{register_bitfields, Bitfield, FieldValue};

use crate::pager::AttributeField::OnDemand;
use core::fmt::{Debug, Formatter};
use core::mem;
use core::ops::{Index, IndexMut};

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
    const OUTPUT_MASK: usize = ((1 << (1 + 47 - 12)) - 1) << 12;

    pub fn null() -> Self {
        Self(0)
    }

    pub fn get(self) -> PageTableEntryType {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        0 != self.0 & 1
    }

    pub const fn is_null(self) -> bool {
        0 == self.0
    }

    pub fn next_level_table_address(self) -> PhysAddr {
        PhysAddr::at((self.0 as usize) & Self::OUTPUT_MASK)
    }

    pub const fn is_table(self, level: u8) -> bool {
        level < 3 && self.0 & 0b10 != 0
    }

    pub fn demand_page(&mut self, phys_addr: PhysAddr) -> Self {
        self.0 |= (phys_addr.get() & Self::OUTPUT_MASK) as u64 | 1;
        *self
    }

    pub const fn is_same_permissions(self, other: PageTableEntry) -> bool {
        (self.0 ^ other.0) | (Self::OUTPUT_MASK as PageTableEntryType) == 0
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
            Reserved = 0b0,                          // Use PageBlockDescriptor
            Table = 0b1
        ],
        Valid OFFSET(0) NUMBITS(1) []
    ]
}

pub type TableDescriptor = Bitfield<PageTableEntryType, TableDescriptorFields::Register>;
type TableDescriptorMask = FieldValue<PageTableEntryType, TableDescriptorFields::Register>;

impl Debug for TableDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use TableDescriptorFields::*;
        write!(
            f,
            "Next-level table: {:?} (",
            self.next_level_table_address(),
        )?;

        match self.read_as_enum(APTable) {
            Some(APTable::Value::NoEffect) => {}
            Some(APTable::Value::PrivOnly) => write!(f, " PrivOnly")?,
            Some(APTable::Value::ReadOnly) => write!(f, " ReadOnly")?,
            Some(APTable::Value::PrivReadOnly) => write!(f, " PrivReadOnly")?,
            None => unreachable!(),
        }
        if self.is_set(UXNTable) {
            write!(f, " UXNTable")?;
        }
        if self.is_set(PXNTable) {
            write!(f, " PXNTable")?;
        }
        write!(f, " )")
    }
}

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
    pub fn new_entry(maybe_phys_addr: Option<PhysAddr>, attributes: Attributes) -> Self {
        use TableDescriptorFields::*;

        let mut field = TableDescriptorMask::from(attributes);
        field += Type::Table;
        match maybe_phys_addr {
            Some(phys_addr) => {
                field +=
                    Valid::SET + NextLevelTableAddress.val(phys_addr.page() as PageTableEntryType);
                Self::from(field)
            }
            None => {
                assert!(attributes.is_set(OnDemand));
                let page = PageBlockDescriptorMask::from(attributes);
                let mut result = Self::new(page.value);
                result.modify(field);
                result
            }
        }
    }

    /// Create a new table descriptor with the same security as the entry in the parent table.
    pub fn new_branch(phys_addr_table: PhysAddr, mut parent_entry: Self) -> Self {
        use TableDescriptorFields::*;
        parent_entry
            .modify(NextLevelTableAddress.val(phys_addr_table.page() as PageTableEntryType));
        parent_entry
    }

    /// Extract table address at next level.
    pub fn next_level_table_address(self) -> PhysAddr {
        use TableDescriptorFields::*;
        PhysAddr::at((self.read(NextLevelTableAddress) * PAGESIZE_BYTES as u64) as usize)
    }
}

impl From<TableDescriptor> for PageTableEntry {
    fn from(desc: TableDescriptor) -> Self {
        Self(desc.get())
    }
}

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
            Page = 0b1,
            Block = 0b0
        ],
        Valid OFFSET(0) NUMBITS(1) []
    ]
}

pub type PageBlockDescriptor = Bitfield<PageTableEntryType, PageBlockDescriptorFields::Register>;
type PageBlockDescriptorMask = FieldValue<PageTableEntryType, PageBlockDescriptorFields::Register>;

impl Debug for PageBlockDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use super::mair::MAIR;
        use PageBlockDescriptorFields::*;

        write!(f, "OA: {:?} (", self.output_address())?;
        if self.is_set(UXN) {
            write!(f, " UXN")?;
        }
        if self.is_set(PXN) {
            write!(f, " PXN")?;
        }
        if self.is_set(Contiguous) {
            write!(f, " Contig")?;
        }
        if self.is_set(nG) {
            write!(f, " nG")?;
        }
        if self.is_set(AF) {
            write!(f, " AF")?;
        }
        match self.read_as_enum(SH) {
            Some(SH::Value::NonShareable) => write!(f, " NonShareable")?,
            Some(SH::Value::OuterShareable) => write!(f, " OuterShareable")?,
            Some(SH::Value::InnerShareable) => write!(f, " InnerShareable")?,
            None => unreachable!(),
        }
        match self.read_as_enum(AP) {
            Some(AP::Value::PrivOnly) => write!(f, " PrivOnly")?,
            Some(AP::Value::ReadWrite) => write!(f, " ReadWrite")?,
            Some(AP::Value::PrivReadOnly) => write!(f, " PrivReadOnly")?,
            Some(AP::Value::ReadOnly) => write!(f, " ReadOnly")?,
            None => unreachable!(),
        }
        write!(f, " {:?}", MAIR::from(self.read(AttrIndx)))?;
        write!(f, " )")
    }
}

impl From<Attributes> for PageBlockDescriptorMask {
    fn from(attributes: Attributes) -> Self {
        use super::mair::MAIR;
        use AttributeField::*;
        use PageBlockDescriptorFields::*;

        const ATTRIBUTE_FIELD_MAP: &[(bool, AttributeField, PageBlockDescriptorMask)] = &[
            (false, UserExec, UXN::SET),
            (false, KernelExec, PXN::SET),
            (true, Device, AF::SET),
            (true, Accessed, AF::SET),
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
            (false, false, false, false) => {
                // presumably execute-only
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
        maybe_output_addr: Option<PhysAddr>,
        attributes: Attributes,
        contiguous: bool,
    ) -> Self {
        use PageBlockDescriptorFields::*;

        let mut field = PageBlockDescriptorMask::from(attributes);
        if let Some(output_addr) = maybe_output_addr {
            field += Valid::SET + OutputAddress.val(output_addr.page() as PageTableEntryType);
        }
        if contiguous {
            field += Contiguous::SET;
        }
        if level == 3 {
            field += Type::Page;
        }
        Self::from(field)
    }

    /// Create a new L3 page descriptor with the same security as the entry in the parent table.
    pub fn new_leaf(phys_addr_output: PhysAddr, parent_entry: TableDescriptor) -> Self {
        use PageBlockDescriptorFields::*;
        let mut result = Self::from(parent_entry);
        result.modify(OutputAddress.val(phys_addr_output.page() as PageTableEntryType));
        result
    }

    /// Extract the output address of the physical page frame.
    pub fn output_address(self) -> PhysAddr {
        PhysAddr::at(
            (self.read(PageBlockDescriptorFields::OutputAddress) * PAGESIZE_BYTES as u64) as usize,
        )
    }

    /// Remove extraneous bits (for testing)
    #[doc(hidden)]
    fn _cleanse_table_bits(self) -> Self {
        const _PAGE_BLOCK_MASK: PageTableEntryType = (1 << 58) - 1;
        Self::new(self.get() & _PAGE_BLOCK_MASK)
    }
}

impl From<PageBlockDescriptor> for PageTableEntry {
    fn from(desc: PageBlockDescriptor) -> Self {
        Self(desc.get())
    }
}

impl From<PageTableEntry> for PageBlockDescriptor {
    fn from(pte: PageTableEntry) -> Self {
        Self::new(pte.0)
    }
}

impl From<TableDescriptor> for PageBlockDescriptor {
    fn from(table_desc: TableDescriptor) -> Self {
        const FIELD_MASK: PageTableEntryType = (0xF << 51) | (0xFFE);
        Self::new(table_desc.get() & FIELD_MASK)
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
        let result = TableDescriptor::new_entry(Some(next_level_table_addr), attributes);
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
        let result =
            PageBlockDescriptor::new_entry(level, Some(output_addr), attributes, contiguous);
        trace!("{:x}", result.get());
        assert_eq!(0x70000001230603, result.get());
    }

    #[test]
    fn test_page_block_descriptor_mask() {
        use crate::pager::Attributes;
        let mut field = PageBlockDescriptorMask::from(Attributes::DEVICE);

        use PageBlockDescriptorFields::*;
        field += Contiguous::SET + Dirty::SET + nG::SET + Valid::SET;

        let result = PageBlockDescriptor::from(field);
        trace!("{:x}", result.get());
        assert_eq!(0x78000000000e01, result.get())
    }

    #[test]
    fn test_demand_page_desc() {
        let attributes = Attributes::KERNEL_DATA;
        let table_desc = TableDescriptor::new_entry(None, attributes);
        let phys_addr = PhysAddr::at(0x1234_9000);
        let page_desc =
            PageBlockDescriptor::from(PageTableEntry::from(table_desc).demand_page(phys_addr))
                ._cleanse_table_bits();
        let page_desc_direct =
            PageBlockDescriptor::new_entry(3, Some(phys_addr), attributes, false);
        dbg!(table_desc);
        dbg!(page_desc);
        dbg!(page_desc_direct);
        assert_eq!(page_desc.get(), page_desc_direct.get());
    }
}
