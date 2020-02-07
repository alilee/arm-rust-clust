use super::desc;
use crate::pager::range::attrs::Attributes;
use desc::{PageBlockDescriptor, TableDescriptor};

use core::fmt::{Debug, Error, Formatter};
use core::panic;

#[derive(Copy, Clone)]
pub struct TranslationAttributes(TableDescriptor, PageBlockDescriptor);

impl TranslationAttributes {
    pub const fn new(table_desc: TableDescriptor, page_desc: PageBlockDescriptor) -> Self {
        Self(table_desc, page_desc)
    }

    pub const fn table_desc(&self) -> TableDescriptor {
        self.0
    }

    pub const fn pageblock_desc(&self) -> PageBlockDescriptor {
        self.1
    }
}

impl From<&TableDescriptor> for TranslationAttributes {
    fn from(pte: &TableDescriptor) -> Self {
        use desc::PageBlockDescriptorFields::Valid;
        Self(*pte, PageBlockDescriptor::new_bitfield(Valid::CLEAR))
    }
}

impl From<Attributes> for TranslationAttributes {
    fn from(attrs: Attributes) -> Self {
        use crate::pager::range::attrs::AttributeFields;
        use desc::PageBlockDescriptorFields;
        use desc::TableDescriptorFields;

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

        page_desc_fields +=
            PageBlockDescriptorFields::AF::SET + PageBlockDescriptorFields::SH::OuterShareable;

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
        )
    }
}

impl From<&PageBlockDescriptor> for TranslationAttributes {
    fn from(pte: &PageBlockDescriptor) -> Self {
        use desc::TableDescriptorFields::Valid;
        Self(TableDescriptor::new_bitfield(Valid::CLEAR), *pte)
    }
}

impl Debug for TranslationAttributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}/{:?}", self.0, self.1)?;
        Ok(())
    }
}
