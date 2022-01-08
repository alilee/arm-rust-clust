// SPDX-License-Identifier: Unlicense

//! Type-checked virtual addresses.
//!
//! ```rust
//! let a = 3;
//! ```

use super::{Addr, AddrRange, PhysAddr, PhysAddrRange, PAGESIZE_BYTES};

use core::fmt::{Debug, Error, Formatter};

/// A local (if kernel range), or a cluster-wide (if low range) virtual address.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct VirtAddr(usize);

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "VirtAddr(0x{:016x})", self.0)
    }
}

impl Addr<VirtAddr, VirtAddrRange> for VirtAddr {
    fn get(&self) -> usize {
        self.0
    }

    fn at(addr: usize) -> Self {
        Self(addr)
    }
}

impl VirtAddr {
    /// Construct from a reference to a linker symbol.
    pub fn from_linker_symbol(sym: *const u8) -> Self {
        Self(sym as usize)
    }

    /// Construct bottom of virtual address range.
    pub const fn null() -> Self {
        Self(0)
    }

    /// Create a virtual address from a physical address.
    ///
    /// UNSAFE: virtual address can only be derefed when identity mapping is in place
    /// before paging is enabled or if physical memory is identity mapped.
    pub unsafe fn identity_mapped(phys_addr: PhysAddr) -> Self {
        Self(phys_addr.get())
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

    /// Extract the page table entry number at a given level offset
    pub const fn get_page_table_entry(self, width: usize, offset: usize) -> usize {
        (self.0 >> offset) & ((1 << width) - 1)
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
            "VirtAddr(0x{:08x}..0x{:08x}, 0x{:09x})",
            self.base.0,
            self.base.0 + self.length,
            self.length
        )
    }
}

impl AddrRange<VirtAddr, VirtAddrRange> for VirtAddrRange {
    fn new(base: VirtAddr, length: usize) -> Self {
        Self { base, length }
    }

    fn base(&self) -> VirtAddr {
        self.base
    }

    fn length(&self) -> usize {
        self.length
    }
}

impl VirtAddrRange {
    /// Create a range from static refs.
    pub fn from_linker_symbols(sym_base: *const u8, sym_top: *const u8) -> Self {
        let base = VirtAddr::from_linker_symbol(sym_base);
        let top = VirtAddr::from_linker_symbol(sym_top);
        Self {
            base,
            length: top.0 - base.0,
        }
    }

    /// Identity mapped from physical address range.
    pub unsafe fn identity_mapped(phys_addr_range: PhysAddrRange) -> Self {
        Self::new(
            VirtAddr::identity_mapped(phys_addr_range.base()),
            phys_addr_range.length(),
        )
    }

    /// Length of the range in bytes.
    pub const fn length_in_pages(&self) -> usize {
        (self.length + PAGESIZE_BYTES - 1) / PAGESIZE_BYTES
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
        assert_eq!(0x45, base.get_page_table_entry(8, 16));
        let top = base.increment(0x1_0000);
        assert_eq!(0x1_0000, base.offset_below(top));

        let _base1: fn() -> ! = virt_id.into();
        let _base2: *mut u32 = virt_id.into();
        let _base3: *const () = virt_id.into();
        let page = crate::pager::Page::new();
        let _base4 = VirtAddr::from(&page);

        unsafe {
            let _base5 = virt_id.as_ref::<u8>();
            let _base6 = virt_id.as_mut_ref::<u8>();
        }

        static SYM: u8 = 43;
        VirtAddr::from_linker_symbol(&SYM);
    }

    #[test]
    fn virt_addr_range() {
        let base = VirtAddr(0x345_0000);
        let range = VirtAddrRange::new(base, 0x1_0000);
        assert_eq!(base, range.base());
        assert_eq!(0x1_0000, range.length());
        assert_eq!(0x10, range.length_in_pages());

        let phys_addr_range = PhysAddrRange::new(PhysAddr::at(0x1000_0000), 0x1000);
        unsafe {
            let range = VirtAddrRange::identity_mapped(phys_addr_range);
            assert_eq!(
                range.base(),
                VirtAddr::identity_mapped(phys_addr_range.base())
            );
            assert_eq!(range.length(), phys_addr_range.length());
        }
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
