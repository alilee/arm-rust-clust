// SPDX-License-Identifier: Unlicense

//! Understanding of physical layout on reset

use crate::pager::{
    Addr, FixedOffset, PhysAddr, PhysAddrRange, Translate, VirtAddr, VirtAddrRange,
};

pub fn kernel_offset() -> FixedOffset {
    FixedOffset::new(
        PhysAddr::at(0x4008_0000),
        VirtAddr::at(0xffff_ff82_0008_0000),
    )
}

/// Kernel boot image (using linker symbols)
pub fn boot_image() -> PhysAddrRange {
    extern "C" {
        static image_base: u8;
        static image_end: u8;
    }
    unsafe {
        kernel_offset().translate_range(VirtAddrRange::from_linker_symbols(&image_base, &image_end))
    }
}

/// Text section of the kernel boot image (using linker symbols)
pub fn text_image() -> PhysAddrRange {
    extern "C" {
        static text_base: u8;
        static text_end: u8;
    }
    unsafe {
        kernel_offset().translate_range(VirtAddrRange::from_linker_symbols(&text_base, &text_end))
    }
}

/// Read-ony data section of the kernel boot image (using linker symbols)
pub fn static_image() -> PhysAddrRange {
    extern "C" {
        static static_base: u8;
        static static_end: u8;
    }
    unsafe {
        kernel_offset().translate_range(VirtAddrRange::from_linker_symbols(
            &static_base,
            &static_end,
        ))
    }
}

/// Zero initialised section of the kernel boot image (using linker symbols)
pub fn bss_image() -> PhysAddrRange {
    extern "C" {
        static bss_base: u8;
        static bss_end: u8;
    }
    unsafe {
        kernel_offset().translate_range(VirtAddrRange::from_linker_symbols(&bss_base, &bss_end))
    }
}

/// Data section of the kernel boot image (using linker symbols)
pub fn data_image() -> PhysAddrRange {
    extern "C" {
        static data_base: u8;
        static data_end: u8;
    }
    unsafe {
        kernel_offset().translate_range(VirtAddrRange::from_linker_symbols(&data_base, &data_end))
    }
}

/// Stack section of the kernel boot image (using linker symbols)
pub fn stack_range() -> PhysAddrRange {
    extern "C" {
        static stack_base: u8;
        static stack_end: u8;
    }
    unsafe {
        kernel_offset().translate_range(VirtAddrRange::from_linker_symbols(&stack_base, &stack_end))
    }
}
