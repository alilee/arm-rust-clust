use super::table::{entry, TranslationAttributes};
use super::{VirtAddr, VirtAddrRange};
use entry::{PageDescriptor, PageDescriptorFields::*};
use entry::{TableDescriptor, TableDescriptorFields::*};

const GB: usize = 1024 * 1024 * 1024;

pub fn kernel_attributes() -> TranslationAttributes {
    let table_attrs = TableDescriptor::new_bitfield(
        APTable::PrivOnly   // EL0 access forbidden
        + UXNTable::SET               // EL0 execution forbidden lower levels
        + PXNTable::CLEAR, // PXN NOT overridden
    );

    let page_attrs = PageDescriptor::new_bitfield(
        UXN::SET                                    // EL0 execution forbidden
            + PXN::CLEAR                                     // EL1 execution allowed
            + Contiguous::CLEAR                              // not contiguous
            + nG::CLEAR                                      // global for all ASIDS
            + AF::SET                                        // accessed
            + SH::OuterShareable                             // Outer Shareable
            + AP::PrivOnly                                   // EL1:RW EL0:none
            + AttrIndx::MemoryWriteThrough, // WriteThrough
    );

    TranslationAttributes::new(table_attrs, page_attrs)
}

pub fn device_attributes() -> TranslationAttributes {
    let table_attrs = TableDescriptor::new_bitfield(
        APTable::PrivOnly                          // EL0 access forbidden
            + UXNTable::SET                                  // EL0 execution forbidden lower levels
            + PXNTable::SET, // PXN overridden no execution
    );

    let page_attrs = PageDescriptor::new_bitfield(
        UXN::SET                                    // EL0 execution forbidden
            + PXN::SET                                       // EL1 execution allowed
            + Contiguous::CLEAR                              // not contiguous
            + nG::CLEAR                                      // global for all ASIDS
            + AF::SET                                        // accessed
            + SH::OuterShareable                             // ignored behind MAIR
            + AP::PrivOnly                                   // EL1:RW EL0:none
            + AttrIndx::DeviceStronglyOrdered, // WriteThrough
    );

    TranslationAttributes::new(table_attrs, page_attrs)
}

pub fn ram_attributes() -> TranslationAttributes {
    let table_attrs = TableDescriptor::new_bitfield(
        APTable::PrivOnly                           // EL0 access forbidden
            + UXNTable::SET                              // EL0 execution forbidden lower levels
            + PXNTable::SET, // PXN NOT overridden
    );

    let page_attrs = PageDescriptor::new_bitfield(
        UXN::SET                                    // EL0 execution forbidden
            + PXN::SET                                       // EL1 execution allowed
            + Contiguous::CLEAR                              // not contiguous
            + nG::CLEAR                                      // global for all ASIDS
            + AF::SET                                        // accessed
            + SH::OuterShareable                             // Outer Shareable
            + AP::PrivOnly                                   // EL1:RW EL0:none
            + AttrIndx::MemoryWriteThrough, // WriteThrough
    );

    TranslationAttributes::new(table_attrs, page_attrs)
}

pub const RAM: VirtAddrRange =
    VirtAddrRange::new_const(VirtAddr::new_const(super::table::UPPER_VA_BASE), 4 * GB);

pub const IMAGE: VirtAddrRange = VirtAddrRange::after(RAM, 1 * GB);

pub const DEVICE: VirtAddrRange = VirtAddrRange::after(IMAGE, 1 * GB);
