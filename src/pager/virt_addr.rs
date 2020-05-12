// SPDX-License-Identifier: Unlicense

//! Type-checked virtual addresses.

use super::PhysAddr;

use core::fmt::{Debug, Error, Formatter};

/// A local (if kernel range), or a cluster-wide (if low range) virtual address.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct VirtAddr(usize);

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "VirtAddr(0x{:016x})", self.0)
    }
}

impl VirtAddr {
    /// Construct at literal address.
    pub const fn at(addr: usize) -> Self {
        Self(addr)
    }

    /// Create a virtual address from a physical address.
    ///
    /// UNSAFE: virtual address can only be derefed when identity mapping is in place
    /// before paging is enabled or if physical memory is identity mapped.
    pub const unsafe fn identity_mapped(phys_addr: PhysAddr) -> Self {
        Self(phys_addr.get())
    }

    /// A virtual address that is higher than this by a given number of bytes.
    pub const fn increment_to(self, higher: VirtAddr) -> usize {
        // assert!(higher.0 >= self.0); # checked in subtraction
        higher.0 - self.0
    }

    /// A virtual address that is higher than this by a given number of bytes.
    pub const fn increment(self, offset: usize) -> Self {
        Self(self.0 + offset)
    }

    /// Create a virtual address range that spans from this address up by length.
    pub const fn extend(self, length: usize) -> VirtAddrRange {
        VirtAddrRange::new(self, length)
    }
}

impl core::convert::Into<*mut u32> for VirtAddr {
    fn into(self) -> *mut u32 {
        self.0 as *mut u32
    }
}

impl core::convert::Into<*const ()> for VirtAddr {
    fn into(self) -> *const () {
        self.0 as *const ()
    }
}

impl core::convert::Into<fn() -> !> for VirtAddr {
    fn into(self) -> fn() -> ! {
        use core::mem;
        unsafe { mem::transmute::<*const (), fn() -> !>(self.into()) }
    }
}

/// A virtual address range.
#[derive(Copy, Clone)]
pub struct VirtAddrRange {
    base: VirtAddr,
    length: usize,
}

impl Debug for VirtAddrRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "VirtAddr(0x{:08x}..0x{:08x}, 0x{:08x})",
            self.base.0,
            self.base.0 + self.length,
            self.length
        )
    }
}

impl VirtAddrRange {
    /// Construct from base and length.
    pub const fn new(base: VirtAddr, length: usize) -> Self {
        Self {
            base,
            length,
        }
    }
}
