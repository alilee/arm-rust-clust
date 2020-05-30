// SPDX-License-Identifier: Unlicense

//! Type-checked virtual addresses.
//!
//! ```rust
//! sandwich;
//! let virt_addr = VirtAddr::nullddd();
//! assert!(false);
//! ```

use super::PhysAddr;

use core::fmt::{Debug, Error, Formatter};
use crate::pager::PAGESIZE_BYTES;

/// A local (if kernel range), or a cluster-wide (if low range) virtual address.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct VirtAddr(usize);

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "VirtAddr(0x{:016x})", self.0)
    }
}

impl VirtAddr {
    /// Construct bottom of virtual address range.
    pub const fn null() -> Self {
        Self(0)
    }
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

    /// A virtual address that is lower than this by a given number of bytes.
    pub const fn decrement(self, offset: usize) -> Self {
        Self(self.0 - offset)
    }

    /// A virtual address that is higher than this by a given number of bytes.
    pub const fn increment(self, offset: usize) -> Self {
        Self(self.0 + offset)
    }

    /// Create a virtual address range that spans from this address up by length.
    pub const fn extend(self, length: usize) -> VirtAddrRange {
        VirtAddrRange::new(self, length)
    }

    /// Nearest higher address with given alignment.
    pub const fn align_up(self, align: usize) -> Self {
        assert!(align.is_power_of_two());
        Self(self.0 + (align - 1) & !(align - 1))
    }

    /// Get the offset from memory base in bytes.
    pub const fn get(self) -> usize {
        self.0
    }

    /// Consume the virtual address as reference.
    ///
    /// UNSAFE: Virtual address must be valid and aligned for T,
    /// and have sufficient lifetime.
    #[allow(unused_unsafe)]
    pub const unsafe fn as_ref<T>(self) -> &'static T {
        use core::mem;
        let e = self.0 as *const T;
        unsafe { mem::transmute::<*const T, &'static T>(e) }
    }

    /// Consume the virtual address as mutable reference.
    ///
    /// UNSAFE: Virtual address must be valid and aligned for T,
    /// and have sufficient lifetime.
    #[allow(unused_unsafe)]
    pub const unsafe fn as_mut_ref<T>(self) -> &'static mut T {
        use core::mem;
        let e = self.0 as *mut T;
        unsafe { mem::transmute::<*mut T, &'static mut T>(e) }
    }
}

impl<T> core::convert::Into<*const T> for VirtAddr {
    fn into(self) -> *const T {
        self.0 as *const T
    }
}

impl<T> core::convert::Into<*mut T> for VirtAddr {
    fn into(self) -> *mut T {
        self.0 as *mut T
    }
}

impl core::convert::Into<fn() -> !> for VirtAddr {
    fn into(self) -> fn() -> ! {
        use core::mem;
        unsafe { mem::transmute::<*const (), fn() -> !>(self.into()) }
    }
}

impl<T> core::convert::From<&T> for VirtAddr {
    fn from(t: &T) -> Self {
        Self::at(t as *const T as usize)
    }
}

/// A virtual address range.
#[derive(Copy, Clone, PartialEq)]
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
        Self { base, length }
    }

    /// Range between two addresses.
    pub const fn between(base: VirtAddr, top: VirtAddr) -> Self {
        // assert!(low < high); # depends on checked subtraction in offset_from
        Self {
            base,
            length: base.increment_to(top),
        }
    }

    /// Range same base different length.
    pub const fn resize(self, length: usize) -> VirtAddrRange {
        VirtAddrRange::new(self.base, length)
    }

    /// Get the base of the range.
    pub const fn base(self) -> VirtAddr {
        self.base
    }

    /// Length of the range in bytes.
    pub const fn length(self) -> usize {
        self.length
    }

    /// Length of the range in bytes.
    pub const fn length_in_pages(&self) -> usize {
        (self.length + PAGESIZE_BYTES - 1) / PAGESIZE_BYTES
    }

    /// Get the top of the range.
    pub const fn top(self: &Self) -> VirtAddr {
        VirtAddr(self.base.0 + self.length)
    }

    /// Length of the range in bytes.
    pub fn intersection(&self, other: &Self) -> VirtAddrRange {
        use core::cmp::{max, min};

        let base = max(self.base.0, other.base.0);
        let low_top = min(self.top().0, other.top().0);
        let top = min(self.top().0, other.top().0);
        if base > low_top {
            return VirtAddrRange::new(VirtAddr::null(), 0);
        }
        VirtAddrRange {
            base: VirtAddr(base),
            length: (top - base) as usize,
        }
    }

    /// Increment the range by its length.
    pub fn step(self: &Self) -> Self {
        Self {
            base: self.base.increment(self.length),
            length: self.length,
        }
    }

    /// True iff the range extends to at least the base and top of other.
    pub fn covers(&self, other: &Self) -> bool {
        self.base <= other.base && self.top() >= other.top()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virt_addr() {
        let null = VirtAddr::null();
        assert_eq!(0, null.get());

        let phys_addr = PhysAddr::at(0x345_0000);
        let virt_id = unsafe { VirtAddr::identity_mapped(phys_addr) };
        let base = VirtAddr(0x345_0000);
        let high = base.increment(0x1_0000);
        assert_eq!(0x1_0000, base.increment_to(high));

        let _base1: fn() -> ! = virt_id.into();
        let _base2: *mut u32 = virt_id.into();
        let _base3: *const () = virt_id.into();
        let page = crate::pager::Page::new();
        let _base4 = VirtAddr::from(&page);

        unsafe {
            let _base5 = virt_id.as_ref::<u8>();
            let _base6 = virt_id.as_mut_ref::<u8>();
        }
    }

    #[test]
    fn virt_addr_range() {
        let base = VirtAddr(0x345_0000);
        let range = base.extend(0x1_0000);
        assert_eq!(base, range.base());
        assert_eq!(0x1_0000, range.length());
        assert_eq!(0x10, range.length_in_pages());
    }

    #[test]
    fn intersection() {
        let lower = VirtAddrRange::between(VirtAddr(0x345_0000), VirtAddr(0x545_0000));
        let higher = VirtAddrRange::between(VirtAddr(0x445_0000), VirtAddr(0x645_0000));
        let intersection = VirtAddrRange::between(VirtAddr(0x445_0000), VirtAddr(0x545_0000));
        assert_eq!(intersection, lower.intersection(&higher));
        assert_eq!(intersection, higher.intersection(&lower));
        assert_eq!(lower, lower.intersection(&lower));
        let disjoint = VirtAddrRange::between(VirtAddr(0x745_0000), VirtAddr(0x800_0000));
        assert_eq!(0, lower.intersection(&disjoint).length());

        assert!(!lower.covers(&higher));
        assert!(lower.covers(&intersection));
        assert!(higher.covers(&intersection));
    }

    #[test]
    fn step() {
        let lower = VirtAddrRange::between(VirtAddr(0x345_0000), VirtAddr(0x545_0000));
        let next = VirtAddrRange::between(VirtAddr(0x545_0000), VirtAddr(0x745_0000));
        assert_eq!(next, lower.step());
    }
}
