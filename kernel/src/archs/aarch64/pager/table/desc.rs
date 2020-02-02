use super::attrs;
use super::mair;
use super::{PageTableEntry, PageTableEntryType};
use attrs::TranslationAttributes;
use mair::MAIR;

use crate::pager::PhysAddr;

use core::fmt::{Debug, Error, Formatter};
use register::{register_bitfields, FieldValue, LocalRegisterCopy};

register_bitfields! {
    u64,
    TableDescriptorFields [
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
        Type OFFSET(1) NUMBITS(1) [],
        Valid OFFSET(0) NUMBITS(1) []
    ]
}

register_bitfields! {
    u64,
    PageBlockDescriptorFields [
        Available OFFSET(55) NUMBITS(9) [],
        UXN OFFSET(54) NUMBITS(1) [],                      // Unprivileged Execute Never
        PXN OFFSET(53) NUMBITS(1) [],                      // Privileged Execute Never
        Contiguous OFFSET(52) NUMBITS(1) [],               // One of a contiguous set of entries
        OutputAddress OFFSET(12) NUMBITS(35) [],
        nG OFFSET(11) NUMBITS(1) [],                       // Not Global - all or current ASID
        AF OFFSET(10) NUMBITS(1) [],                       // Access flag
        SH OFFSET(8) NUMBITS(2) [                          // Shareability
            NonShareable = 0b00,
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],
        AP OFFSET(6) NUMBITS(2) [                        // Data access permissions
            PrivOnly = 0b00,
            ReadWrite = 0b01,
            PrivReadOnly = 0b10,
            ReadOnly = 0b11
        ],
        AttrIndx OFFSET(2) NUMBITS(3) [                  // Memory attributes index for MAIR_ELx
            DeviceStronglyOrdered = 0,
            MemoryWriteThrough = 1
        ],
        Type OFFSET(1) NUMBITS(1) [],
        Valid OFFSET(0) NUMBITS(1) []
    ]
}

#[derive(Copy, Clone)]
pub struct TableDescriptor(PageTableEntryType);

type TableDescReg = TableDescriptorFields::Register;
type TableDescLocal = LocalRegisterCopy<PageTableEntryType, TableDescReg>;

impl TableDescriptor {
    pub fn new_entry(pt: PhysAddr, attributes: TranslationAttributes) -> Self {
        use TableDescriptorFields::*;

        let nlta = pt.get() >> 12;
        let field = Valid::SET + Type::SET + NextLevelTableAddress.val(nlta as u64);
        let value = (attributes.table_desc().0 & !field.mask) | field.value;
        Self(value)
    }

    pub const fn new_bitfield(field_value: FieldValue<u64, TableDescReg>) -> Self {
        Self(field_value.value)
    }

    pub fn is_valid(&self) -> bool {
        let r = TableDescLocal::new(self.0);
        r.is_set(TableDescriptorFields::Valid)
    }

    pub fn next_level_table_address(&self) -> PhysAddr {
        let field = TableDescriptorFields::NextLevelTableAddress;
        PhysAddr::new((self.0 & (field.mask << field.shift)) as usize)
    }

    pub fn get(&self) -> PageTableEntryType {
        self.0
    }
}

impl From<PageTableEntry> for TableDescriptor {
    fn from(pte: PageTableEntry) -> Self {
        Self(pte.get())
    }
}

impl Debug for TableDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.next_level_table_address().get() != 0 {
            write!(
                f,
                "Next-level table: {:?} ",
                self.next_level_table_address()
            )?;
        }

        use TableDescriptorFields::*;
        let attrs = TableDescLocal::new(self.0);
        write!(f, "(")?;
        write!(f, "APTable({:02b})", attrs.read(APTable))?;
        if attrs.is_set(UXNTable) {
            write!(f, " UXNTable")?;
        }
        if attrs.is_set(PXNTable) {
            write!(f, " PXNTable")?;
        }
        write!(f, ")")?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct PageBlockDescriptor(PageTableEntryType);

type PageBlockDescReg = PageBlockDescriptorFields::Register;
type PageBlockDescLocal = LocalRegisterCopy<PageTableEntryType, PageBlockDescReg>;

impl PageBlockDescriptor {
    pub fn new_entry(
        level: u8,
        output_addr: PhysAddr,
        attributes: TranslationAttributes,
        contiguous: bool,
    ) -> Self {
        use PageBlockDescriptorFields::*;

        let mut field = Valid::SET + OutputAddress.val(output_addr.get() as u64 >> 12);

        if contiguous {
            field += Contiguous::SET;
        } else {
            field += Contiguous::CLEAR;
        }

        if level == 3 {
            field += Type::SET;
        }

        let value = (attributes.pageblock_desc().0 & !field.mask) | field.value;
        Self(value)
    }

    pub const fn new_bitfield(field_value: FieldValue<u64, PageBlockDescReg>) -> Self {
        Self(field_value.value)
    }

    pub fn is_contiguous(&self) -> bool {
        PageBlockDescLocal::new(self.0).is_set(PageBlockDescriptorFields::Contiguous)
    }

    pub fn output_address(&self) -> PhysAddr {
        use PageBlockDescriptorFields::*;

        let field = OutputAddress;
        PhysAddr::new((self.0 & (field.mask << field.shift)) as usize)
    }

    pub fn get(&self) -> PageTableEntryType {
        self.0
    }
}

impl From<PageTableEntry> for PageBlockDescriptor {
    fn from(pte: PageTableEntry) -> Self {
        Self(pte.get())
    }
}

impl Debug for PageBlockDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.output_address().get() != 0 {
            write!(f, "OA: {:?} ", self.output_address(),)?;
        }
        use PageBlockDescriptorFields::*;

        write!(f, "(")?;
        let attrs = PageBlockDescLocal::new(self.0);
        if attrs.is_set(UXN) {
            write!(f, " UXN")?;
        }
        if attrs.is_set(PXN) {
            write!(f, " PXN")?;
        }
        if attrs.is_set(Contiguous) {
            write!(f, " Contig")?;
        }
        if attrs.is_set(nG) {
            write!(f, " nG")?;
        }
        if attrs.is_set(AF) {
            write!(f, " AF")?;
        }
        write!(f, " SH({:02b})", attrs.read(SH))?;
        write!(f, " AP({:02b})", attrs.read(AP))?;
        write!(f, " {:?}", MAIR::from(attrs.read(AttrIndx)))?;
        write!(f, ")")?;
        Ok(())
    }
}
