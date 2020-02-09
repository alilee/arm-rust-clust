use super::desc;
use crate::pager::{attrs, attrs::Attributes};
use desc::{PageBlockDescriptor, TableDescriptor};

use core::fmt::{Debug, Error, Formatter};
use core::panic;

#[derive(Copy, Clone)]
pub struct TranslationAttributes {
    table_desc: TableDescriptor,
    page_desc: PageBlockDescriptor,
    is_provisional: bool,
}

impl TranslationAttributes {
    pub const fn new(
        table_desc: TableDescriptor,
        page_desc: PageBlockDescriptor,
        is_provisional: bool,
    ) -> Self {
        Self {
            table_desc,
            page_desc,
            is_provisional,
        }
    }

    pub fn from_attrs(attrs: Attributes) -> Self {
        Self::from_attrs_inner(attrs, false)
    }

    pub fn from_attrs_provisional(attrs: Attributes) -> Self {
        Self::from_attrs_inner(attrs, true)
    }

    fn from_attrs_inner(attrs: Attributes, is_provisional: bool) -> Self {
        use attrs::AttributeFields;
        use desc::{PageBlockDescriptorFields, TableDescriptorFields};

        let mut table_desc_fields = TableDescriptorFields::Valid::SET;
        let mut page_desc_fields = PageBlockDescriptorFields::Valid::SET;

        if !attrs.is_set(AttributeFields::UserExec) {
            table_desc_fields += TableDescriptorFields::UXNTable::SET;
            page_desc_fields += PageBlockDescriptorFields::UXN::SET;
        }

        if !attrs.is_set(AttributeFields::KernelExec) {
            table_desc_fields += TableDescriptorFields::PXNTable::SET;
            page_desc_fields += PageBlockDescriptorFields::PXN::SET;
        }

        if attrs.is_set(AttributeFields::Block) {
            page_desc_fields += PageBlockDescriptorFields::Contiguous::SET;
        }

        if !attrs.is_provisional() {
            page_desc_fields += PageBlockDescriptorFields::AF::SET;
        }

        page_desc_fields += PageBlockDescriptorFields::SH::OuterShareable;

        match (
            attrs.is_set(AttributeFields::UserRead),
            attrs.is_set(AttributeFields::UserWrite),
            attrs.is_set(AttributeFields::KernelRead),
            attrs.is_set(AttributeFields::KernelWrite),
        ) {
            (true, true, true, true) => {
                page_desc_fields += PageBlockDescriptorFields::AP::ReadWrite
            }
            (false, false, true, true) => {
                page_desc_fields += PageBlockDescriptorFields::AP::PrivOnly
            }
            (true, false, true, false) => {
                page_desc_fields += PageBlockDescriptorFields::AP::ReadOnly
            }
            (false, false, true, false) => {
                page_desc_fields += PageBlockDescriptorFields::AP::PrivReadOnly
            }
            _ => panic!(),
        }

        if attrs.is_set(AttributeFields::Device) {
            page_desc_fields += PageBlockDescriptorFields::AttrIndx::DeviceStronglyOrdered
        } else {
            page_desc_fields += PageBlockDescriptorFields::AttrIndx::MemoryWriteThrough
        }

        TranslationAttributes::new(
            TableDescriptor::new_bitfield(table_desc_fields),
            PageBlockDescriptor::new_bitfield(page_desc_fields),
            attrs.is_set(AttributeFields::Provisional),
        )
    }

    pub const fn table_desc(&self) -> TableDescriptor {
        self.table_desc
    }

    pub const fn pageblock_desc(&self) -> PageBlockDescriptor {
        self.page_desc
    }

    //    pub const fn is_provisional(&self) -> bool {
    //        self.is_provisional
    //    }
}

impl From<&TableDescriptor> for TranslationAttributes {
    fn from(pte: &TableDescriptor) -> Self {
        use desc::PageBlockDescriptorFields::Valid;
        Self::new(*pte, PageBlockDescriptor::new_bitfield(Valid::CLEAR), false)
    }
}

impl From<&PageBlockDescriptor> for TranslationAttributes {
    fn from(pte: &PageBlockDescriptor) -> Self {
        use desc::TableDescriptorFields::Valid;
        Self::new(TableDescriptor::new_bitfield(Valid::CLEAR), *pte, false)
    }
}

impl Debug for TranslationAttributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}/{:?}", self.0, self.1)?;
        Ok(())
    }
}
