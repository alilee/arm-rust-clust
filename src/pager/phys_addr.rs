// SPDX-License-Identifier: Unlicense

use core::fmt::{Debug, Error, Formatter};

/// A local physical address
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(usize);

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), Error> {
        write!(f, "PhysAddr(0x{:08x})", self.0)
    }
}

impl PhysAddr {
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
            Self::new(
                 base,
                top.offset_from(base),
            )
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
}
