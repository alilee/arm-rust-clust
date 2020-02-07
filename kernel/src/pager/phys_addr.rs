use super::Page;
use super::PAGESIZE_BYTES;

use crate::pager::virt_addr::VirtAddr;
use core::fmt::{Debug, Error, Formatter};

/// A local physical address
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(usize);

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "PhysAddr(0x{:08x})", self.0)
    }
}

impl PhysAddr {
    pub fn new(base: usize) -> PhysAddr {
        PhysAddr(base)
    }
    pub const fn new_const(base: usize) -> PhysAddr {
        PhysAddr(base)
    }
    pub fn from_fn(f: fn() -> !) -> PhysAddr {
        PhysAddr(f as *const () as usize)
    }
    pub fn from_linker_symbol(sym: &u8) -> Self {
        Self(sym as *const u8 as usize)
    }

    pub fn align_down(self: &Self, align: usize) -> Self {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        Self(self.0 & !(align - 1))
    }

    pub fn align_up(self: &Self, align: usize) -> Self {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        let result = (self.0 + align - 1) & !(align - 1);
        Self(result)
    }

    pub fn _identity_map(&self) -> *const () {
        self.0 as *const ()
    }

    pub const fn identity_map_mut(&self) -> *mut () {
        self.0 as *mut ()
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub const fn page(&self) -> usize {
        self.0 >> 12
    }

    fn as_ptr(self: &Self) -> *const u8 {
        self.0 as *const u8
    }

    pub fn offset(self: &Self, distance: usize) -> Self {
        Self(self.0 + distance)
    }

    pub unsafe fn get(&self) -> usize {
        self.0
    }
}

impl From<*const u8> for PhysAddr {
    fn from(p: *const u8) -> Self {
        Self(p as usize)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MemOffset(usize);

impl MemOffset {
    pub fn new(ram_base: PhysAddr, kernel_ram_location: VirtAddr) -> Self {
        unsafe { Self(kernel_ram_location.get() - ram_base.get()) }
    }

    pub fn identity() -> Self {
        MemOffset(0)
    }

    pub fn offset(&self, phys_addr: PhysAddr) -> *const Page {
        let va = VirtAddr::id_map(phys_addr);
        unsafe { va.increment(self.0).as_ptr() as *const Page }
    }

    pub fn offset_mut(&self, phys_addr: PhysAddr) -> *mut Page {
        let va = VirtAddr::id_map(phys_addr);
        unsafe { va.increment(self.0).as_ptr() as *mut Page }
    }
}

/// A range in the VA space
#[derive(Copy, Clone)]
pub struct PhysAddrRange {
    base: PhysAddr,
    length: usize,
}

impl PhysAddrRange {
    pub fn new(base: PhysAddr, length: usize) -> Self {
        Self { base, length }
    }

    pub const fn new_const(base: PhysAddr, length: usize) -> Self {
        Self { base, length }
    }

    pub fn pages_bounding(base: PhysAddr, top: PhysAddr) -> Self {
        assert!(base.0 < top.0);
        let top = top.align_up(PAGESIZE_BYTES);
        unsafe {
            let length = top.as_ptr().offset_from(base.as_ptr()) as usize;
            Self { base, length }
        }
    }

    pub fn extend_to_align_to(&self, align: usize) -> Self {
        unsafe {
            let top = self.top().get() + align - 1;
            let aligned_top = top & !(align - 1);
            Self {
                base: self.base.align_down(align),
                length: aligned_top - self.base.get(),
            }
        }
    }

    pub fn pages(self: &Self) -> usize {
        self.length / PAGESIZE_BYTES
    }

    pub fn top(self: &Self) -> PhysAddr {
        // FIXME: Wrapping around?
        PhysAddr(self.base.0 + self.length)
    }

    pub fn base(&self) -> PhysAddr {
        self.base
    }
    pub fn length(&self) -> usize {
        self.length
    }

    pub fn outside(self: &Self, other: &Self) -> bool {
        self.base < other.base || self.top() > other.top()
    }
}

impl Debug for PhysAddrRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "PhysAddr(0x{:08x}..0x{:08x}, 0x{:08x})",
            self.base.0,
            self.top().0,
            self.length
        )
    }
}
