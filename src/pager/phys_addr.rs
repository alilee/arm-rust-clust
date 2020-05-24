// SPDX-License-Identifier: Unlicense


use super::VirtAddr;
use core::fmt::{Debug, Error, Formatter};
use crate::pager::PAGESIZE_BYTES;

/// A local physical address
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(usize);

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), Error> {
        write!(f, "PhysAddr(0x{:08x})", self.0)
    }
}

impl PhysAddr {
    /// Lowest possible phyical address.
    pub const fn null() -> Self {
        Self(0)
    }

    /// At literal address.
    pub const fn at(addr: usize) -> Self {
        Self(addr)
    }

    /// At virtual address, assuming identity mapping.
    pub const fn identity_mapped(virt_addr: VirtAddr) -> Self {
        Self(virt_addr.get())
    }

    /// Number of bytes above reference point.
    pub const fn offset_from(self, base: Self) -> usize {
        // assert!(self.0 > base.0); - relies on checked subtraction to avoid underflow
        self.0 - base.0
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
    pub unsafe fn from_fn(f: fn()) -> Self {
        Self::from_ptr(f as *const u8)
    }

    /// Construct from a reference to a linker symbol.
    pub const fn from_linker_symbol(sym: &u8) -> Self {
        unsafe { Self(sym as *const u8 as usize) }
    }

    /// Get address as an integer.
    pub const fn get(&self) -> usize {
        self.0
    }

    /// Aligned on a byte boundary.
    pub const fn aligned(&self, bytes: usize) -> bool {
        0 == self.0 & (bytes - 1)
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

/// An iterator over byte-sized chunks of a physical address range.
pub struct PhysAddrRangeIterator {
    base: usize,
    length: usize,
    bytes: usize,
}

impl PhysAddrRange {
    /// Get the physical address range of the kernel boot image (using linker symbols)
    pub fn boot_image() -> Self {
        extern "C" {
            static image_base: u8;
            static image_end: u8;
        }
        unsafe {
            let base = PhysAddr::from_linker_symbol(&image_base);
            let top = PhysAddr::from_linker_symbol(&image_end);
            Self::new(base, top.offset_from(base))
        }
    }

    /// Range of length starting at base
    pub const fn new(base: PhysAddr, length: usize) -> Self {
        Self { base, length }
    }

    /// Range between two addresses
    pub const fn between(base: PhysAddr, top: PhysAddr) -> Self {
        // assert!(low < high); # depends on checked subtraction in offset_from
        Self {
            base,
            length: top.offset_from(base),
        }
    }

    /// PhysAddr of the bottom of the range.
    pub const fn base(&self) -> PhysAddr {
        self.base
    }

    /// Length of the range in bytes
    pub const fn length(&self) -> usize {
        self.length
    }

    /// aligned base and top
    pub const fn aligned(&self, bytes: usize) -> bool {
        self.base().aligned(bytes) || (0 == self.length & (bytes - 1))
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

    #[test]
    fn phys_addr() {
        let null = PhysAddr::null();
        assert_eq!(0, null.get());

        let base = PhysAddr(0x345_0000);
        let c: u8 = 42u8;
        static SYM: u8 = 43u8;
        assert_eq!(0x345_0000, base.get());
        assert_eq!(0x1_0000, PhysAddr(0x346_0000).offset_from(base));

        unsafe {
            PhysAddr::from_ptr(&c);
            PhysAddr::from_fn(phys_addr);
        };
        PhysAddr::from_linker_symbol(&SYM);
    }

    #[test]
    fn phys_addr_range() {
        let base = PhysAddr(0x345_0000);
        let _image_range = PhysAddrRange::new(base, 0x1_0000);
        let _boot_image_range = PhysAddrRange::boot_image();
        let top = PhysAddr(0x346_0000);
        let between_range = PhysAddrRange::between(base, top);
        assert_eq!(base, between_range.base());
        assert_eq!(0x1_0000, between_range.length());
    }

    #[test]
    fn alignment() {
        let base = PhysAddr(0x1000_0010);
        assert!(!base.aligned(0x100));
        let top = PhysAddr(0x1000_1000);
        assert!(top.aligned(0x100));
        let range = PhysAddrRange::between(base, top);
        assert!(!range.aligned(0x100));
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
