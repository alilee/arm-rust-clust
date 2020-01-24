mod frames;
mod trans;

use crate::arch;
use log::info;

use core::cmp::{max, min};
use core::fmt::{Debug, Error, Formatter};

const PAGESIZE_BYTES: usize = 4096;
const PAGESIZE_WORDS: usize = PAGESIZE_BYTES / 4;

/// A cluster-wide virtual address
#[derive(Copy, Clone)]
pub struct VirtAddr(u64);

impl VirtAddr {
    pub fn init(i: u64) -> VirtAddr {
        VirtAddr(i)
    }
    pub fn id_map(pa: PhysAddr) -> VirtAddr {
        VirtAddr(pa.0)
    }
    pub fn forward(&self, step: u64) -> VirtAddr {
        VirtAddr(self.0 + step)
    }
    pub fn addr(&self) -> u64 {
        self.0
    }
    pub fn offset(&self, offset: u64) -> VirtAddr {
        VirtAddr(self.0 + offset)
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "VirtAddr(0x{:08x})", self.0)
    }
}

/// A range in the VA space
#[derive(Copy, Clone)]
pub struct VirtAddrRange {
    pub base: VirtAddr,
    pub length: usize,
}

impl VirtAddrRange {
    pub fn id_map(range: PhysAddrRange) -> VirtAddrRange {
        VirtAddrRange {
            base: VirtAddr::id_map(range.base),
            length: range.length,
        }
    }

    pub fn step(self: &Self) -> VirtAddrRange {
        VirtAddrRange {
            base: self.base.forward(self.length as u64),
            length: self.length,
        }
    }

    pub fn top(self: &Self) -> VirtAddr {
        VirtAddr(self.base.0 + self.length as u64)
    }

    pub fn intersection(self: &Self, other: &Self) -> VirtAddrRange {
        assert!(self.top().0 >= other.base.0 || self.base.0 <= other.top().0);
        let base = max(self.base.0, other.base.0);
        let top = min(self.top().0, other.top().0);
        VirtAddrRange {
            base: VirtAddr(base),
            length: (top - base) as usize,
        }
    }
}

impl Debug for VirtAddrRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "VirtAddr(0x{:08x}..0x{:08x}, 0x{:08x})",
            self.base.0,
            self.top().0,
            self.length
        )
    }
}

/// A local physical address
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(u64);

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "PhysAddr(0x{:08x})", self.0)
    }
}

impl PhysAddr {
    pub fn from_linker_symbol(sym: &u8) -> Self {
        Self(sym as *const u8 as u64)
    }

    pub fn align_down(self: &Self, align: usize) -> Self {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        Self(self.0 & !(align as u64 - 1))
    }

    pub fn as_ptr(self: &Self) -> *const u8 {
        self.0 as *const u8
    }

    pub fn offset(self: &Self, distance: u64) -> Self {
        Self(self.0 + distance)
    }
}

impl From<*const u8> for PhysAddr {
    fn from(p: *const u8) -> Self {
        Self(p as u64)
    }
}

/// A range in the VA space
#[derive(Copy, Clone)]
pub struct PhysAddrRange {
    base: PhysAddr,
    length: usize,
}

impl PhysAddrRange {
    pub fn bounded_by(base: PhysAddr, top: PhysAddr) -> Self {
        assert!(base.0 < top.0);
        unsafe {
            let length = top.as_ptr().offset_from(base.as_ptr()) as usize;
            Self { base, length }
        }
    }

    fn pages(self: &Self, page_size: usize) -> usize {
        self.length / page_size
    }

    fn top(self: &Self) -> PhysAddr {
        // FIXME: Wrapping around?
        PhysAddr(self.base.0 + self.length as u64)
    }

    fn outside(self: &Self, other: &Self) -> bool {
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

/// Initialise the system by initialising the submodules and mapping initial memory contents.
pub fn init() {
    info!("initialising");
    arch::pager::init();
}
