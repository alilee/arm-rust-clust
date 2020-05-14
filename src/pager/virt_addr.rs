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

impl<T> core::convert::Into<*mut T> for VirtAddr {
    fn into(self) -> *mut T {
        self.0 as *mut T
    }
}

impl<T> core::convert::Into<*const T> for VirtAddr {
    fn into(self) -> *const T {
        self.0 as *const T
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virt_addr() {
        let phys_addr = PhysAddr::at(0x345_0000);
        let virt_id = unsafe { VirtAddr::identity_mapped(phys_addr) };
        let base = VirtAddr(0x345_0000);
        let high = base.increment(0x1_0000);
        assert_eq!(0x1_0000, base.increment_to(high));
        let _base1: fn() -> ! = virt_id.into();
        let _base2: *mut u32 = virt_id.into();
        let _base3: *const () = virt_id.into();
    }

    #[test]
    fn virt_addr_range() {
        let base = VirtAddr(0x345_0000);
        let _range = base.extend(0x1_0000);
    }
}