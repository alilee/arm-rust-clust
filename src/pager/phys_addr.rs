// SPDX-License-Identifier: Unlicense


use super::{Addr, AddrRange, VirtAddr, PAGESIZE_BYTES};

use core::fmt::{Debug, Error, Formatter};

/// A local physical address
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(usize);

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), Error> {
        write!(f, "PhysAddr(0x{:08x})", self.0)
    }
}

impl Addr<PhysAddr, PhysAddrRange> for PhysAddr {
    /// Get address as an integer.
    fn get(&self) -> usize {
        self.0
    }

    /// At literal address.
    fn at(addr: usize) -> Self {
        Self(addr)
    }
}

impl PhysAddr {
    /// Construct bottom of virtual address range.
    pub const fn null() -> Self {
        Self(0)
    }

    /// At virtual address, assuming identity mapping.
    pub fn identity_mapped(virt_addr: VirtAddr) -> Self {
        Self(virt_addr.get())
    }

    /// Construct from a pointer.
    ///
    /// UNSAFE: pointer must be to physical memory ie. before paging is enabled or under
    /// identity mapping.
    pub unsafe fn from_ptr(p: *const u8) -> Self {
        Self(p as usize)
    }

    /// Construct from a pointer.
    ///
    /// UNSAFE: pointer must be to physical memory ie. before paging is enabled or under
    /// identity mapping.
    pub unsafe fn from_fn(f: fn() -> !) -> Self {
        Self::from_ptr(f as *const u8)
    }

    /// Construct from a reference to a linker symbol.
    pub const fn from_linker_symbol(sym: &u8) -> Self {
        unsafe { Self(sym as *const u8 as usize) }
    }

    /// Page number.
    pub const fn page(&self) -> usize {
        self.0 / PAGESIZE_BYTES
    }
}

/// A physical address range.
#[derive(Copy, Clone)]
pub struct PhysAddrRange {
    base: PhysAddr,
    length: usize,
}

impl Debug for PhysAddrRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "PhysAddr(0x{:08x}..0x{:08x}, 0x{:08x})",
            self.base.0,
            self.base.0 + self.length,
            self.length
        )
    }
}

impl AddrRange<PhysAddr, PhysAddrRange> for PhysAddrRange {
    fn new(base: PhysAddr, length: usize) -> Self {
        Self { base, length }
    }

    fn base(&self) -> PhysAddr {
        self.base
    }

    fn length(&self) -> usize {
        self.length
    }
}

/// An iterator over byte-sized chunks of a physical address range.
pub struct PhysAddrRangeIterator {
    base: usize,
    length: usize,
    bytes: usize,
}

impl PhysAddrRange {
    /// Create a range from static refs.
    fn from_linker_symbols(sym_base: &'static u8, sym_top: &'static u8) -> Self {
        let base = PhysAddr::from_linker_symbol(&sym_base);
        let top = PhysAddr::from_linker_symbol(&sym_top);
        Self::new(base, top.offset_above(base))
    }

    /// Kernel boot image (using linker symbols)
    pub fn boot_image() -> Self {
        extern "C" {
            static image_base: u8;
            static image_end: u8;
        }
        unsafe { Self::from_linker_symbols(&image_base, &image_end) }
    }

    /// Text section of the kernel boot image (using linker symbols)
    pub fn text_image() -> Self {
        extern "C" {
            static text_base: u8;
            static text_end: u8;
        }
        unsafe { Self::from_linker_symbols(&text_base, &text_end) }
    }

    /// ROdata section of the kernel boot image (using linker symbols)
    pub fn static_image() -> Self {
        extern "C" {
            static static_base: u8;
            static static_end: u8;
        }
        unsafe { Self::from_linker_symbols(&static_base, &static_end) }
    }

    /// Data section of the kernel boot image (using linker symbols)
    pub fn data_image() -> Self {
        extern "C" {
            static data_base: u8;
            static data_end: u8;
        }
        unsafe { Self::from_linker_symbols(&data_base, &data_end) }
    }

    /// Length of the range in bytes.
    pub const fn length_in_pages(&self) -> usize {
        (self.length + PAGESIZE_BYTES - 1) / PAGESIZE_BYTES
    }

    /// Iterate over range in chunks of bytes.
    pub const fn chunks(&self, bytes: usize) -> PhysAddrRangeIterator {
        PhysAddrRangeIterator {
            base: self.base.0,
            length: self.length,
            bytes,
        }
    }
}

impl Iterator for PhysAddrRangeIterator {
    type Item = PhysAddr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None
        }
        let result = self.base;
        self.base += self.bytes;
        self.length -= self.bytes;
        Some(PhysAddr(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foo() -> ! {
        unimplemented!()
    }

    #[test]
    fn phys_addr() {
        let null = PhysAddr::null();
        assert_eq!(0, null.get());

        let base = PhysAddr(0x345_0000);
        let c: u8 = 42u8;
        static SYM: u8 = 43u8;
        assert_eq!(0x345_0000, base.get());
        assert_eq!(0x1_0000, PhysAddr(0x346_0000).offset_above(base));

        unsafe {
            PhysAddr::from_ptr(&c);
            PhysAddr::from_fn(foo);
        };
        PhysAddr::from_linker_symbol(&SYM);
    }

    #[test]
    fn phys_addr_range() {
        let base = PhysAddr(0x345_0000);
        let _image_range = PhysAddrRange::new(base, 0x1_0000);
        let _boot_image_range = PhysAddrRange::text_image();
        let top = PhysAddr(0x346_0000);
        let between_range = PhysAddrRange::between(base, top);
        assert_eq!(base, between_range.base());
        assert_eq!(0x1_0000, between_range.length());
    }

    #[test]
    fn alignment() {
        let base = PhysAddr(0x1000_0010);
        assert!(!base.is_aligned(0x100));
        let top = PhysAddr(0x1000_1000);
        assert!(top.is_aligned(0x100));
        let range = PhysAddrRange::between(base, top);
        assert!(!range.is_aligned(0x100));
    }

    #[test]
    fn iterator() {
        let range = PhysAddrRange::between(PhysAddr(0x1000), PhysAddr(0x3000));
        let mut range_iter = range.chunks(0x1000);
        assert_some_eq!(range_iter.next(), PhysAddr(0x1000));
        assert_some_eq!(range_iter.next(), PhysAddr(0x2000));
        assert_none!(range_iter.next());
    }
}
