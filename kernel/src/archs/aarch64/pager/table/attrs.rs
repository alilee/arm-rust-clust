use super::desc;
use crate::pager::range::attrs::Attributes;
use desc::{PageDescriptor, TableDescriptor};

use core::fmt::{Debug, Error, Formatter};
use core::panic;

#[derive(Copy, Clone)]
pub struct TranslationAttributes(TableDescriptor, PageDescriptor);

impl TranslationAttributes {
    pub const fn new(table_desc: TableDescriptor, page_desc: PageDescriptor) -> Self {
        Self(table_desc, page_desc)
    }

    pub const fn table_desc(&self) -> TableDescriptor {
        self.0
    }

    pub const fn page_desc(&self) -> PageDescriptor {
        self.1
    }
}

impl From<&TableDescriptor> for TranslationAttributes {
    fn from(pte: &TableDescriptor) -> Self {
        use desc::PageDescriptorFields::Valid;
        Self(*pte, PageDescriptor::new_bitfield(Valid::CLEAR))
    }
}

impl From<Attributes> for TranslationAttributes {
    fn from(attrs: Attributes) -> Self {
        use crate::pager::range::attrs::AttributeFields;
        use desc::PageDescriptorFields;
        use desc::TableDescriptorFields;

        let mut table_desc_fields = TableDescriptorFields::Valid::SET;
        let mut page_desc_fields = PageDescriptorFields::Valid::SET;

        if !attrs.is_set(AttributeFields::UserExec) {
            table_desc_fields += TableDescriptorFields::UXNTable::SET;
            page_desc_fields += PageDescriptorFields::UXN::SET;
        }

        if !attrs.is_set(AttributeFields::KernelExec) {
            table_desc_fields += TableDescriptorFields::PXNTable::SET;
            page_desc_fields += PageDescriptorFields::PXN::SET;
        }

        page_desc_fields +=
            PageDescriptorFields::AF::SET + PageDescriptorFields::SH::OuterShareable;

        match (
            attrs.is_set(AttributeFields::UserRead),
            attrs.is_set(AttributeFields::UserWrite),
            attrs.is_set(AttributeFields::KernelRead),
            attrs.is_set(AttributeFields::KernelWrite),
        ) {
            (true, true, true, true) => page_desc_fields += PageDescriptorFields::AP::ReadWrite,
            (false, false, true, true) => page_desc_fields += PageDescriptorFields::AP::PrivOnly,
            (true, false, true, false) => page_desc_fields += PageDescriptorFields::AP::ReadOnly,
            (false, false, true, false) => {
                page_desc_fields += PageDescriptorFields::AP::PrivReadOnly
            }
            _ => panic!(),
        }

        if attrs.is_set(AttributeFields::Device) {
            page_desc_fields += PageDescriptorFields::AttrIndx::DeviceStronglyOrdered
        } else {
            page_desc_fields += PageDescriptorFields::AttrIndx::MemoryWriteThrough
        }

        TranslationAttributes::new(
            TableDescriptor::new_bitfield(table_desc_fields),
            PageDescriptor::new_bitfield(page_desc_fields),
        )
    }
}

impl From<&PageDescriptor> for TranslationAttributes {
    fn from(pte: &PageDescriptor) -> Self {
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
