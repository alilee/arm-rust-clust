use super::PAGESIZE_BYTES;

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

    pub fn align_down(self: &Self, align: usize) -> (Self, MemOffset) {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        let result = self.0 & !(align - 1);
        (Self(result), MemOffset((self.0 - result) as isize))
    }
    pub fn align_up(self: &Self, align: usize) -> Self {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        let result = (self.0 + align - 1) & !(align - 1);
        Self(result)
    }

    pub fn identity_map(&self) -> *const () {
        self.0 as *const ()
    }
    pub const fn identity_map_mut(&self) -> *mut () {
        self.0 as *mut ()
    }
    fn as_ptr(self: &Self) -> *const u8 {
        self.0 as *const u8
    }

    pub fn offset(self: &Self, distance: usize) -> Self {
        Self(self.0 + distance)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl From<*const u8> for PhysAddr {
    fn from(p: *const u8) -> Self {
        Self(p as usize)
    }
}

pub struct MemOffset(isize);

impl MemOffset {
    pub fn offset(&self, virt_addr: *const ()) -> *const () {
        let pb = virt_addr as *const u8;
        unsafe { pb.offset(self.0) as *const () }
    }
    pub fn offset_mut(&self, virt_addr: *mut ()) -> *mut () {
        let pb = virt_addr as *mut u8;
        unsafe { pb.offset(self.0) as *mut () }
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
    pub fn bounded_by(base: PhysAddr, top: PhysAddr) -> Self {
        assert!(base.0 < top.0);
        let top = top.align_up(PAGESIZE_BYTES);
        unsafe {
            let length = top.as_ptr().offset_from(base.as_ptr()) as usize;
            Self { base, length }
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
