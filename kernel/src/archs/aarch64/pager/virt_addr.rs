use crate::pager::{PhysAddr, PhysAddrRange};

use core::fmt::{Debug, Error, Formatter};

//register_bitfields! {
//    u64,
//    VirtualAddress [
//        L0Index OFFSET(39) NUMBITS(9) [],
//        L1Index OFFSET(30) NUMBITS(9) [],
//        L2Index OFFSET(21) NUMBITS(9) [],
//        L3Index OFFSET(12) NUMBITS(9) [],
//        PageOffset OFFSET(0) NUMBITS(12) []
//    ]
//}

/// A single-step addition offset between a physical address and a virtual location
#[derive(Copy, Clone)]
pub struct VirtOffset(usize);

impl VirtOffset {
    pub fn init(offset: usize) -> VirtOffset {
        VirtOffset(offset)
    }
    pub const fn init_const(offset: usize) -> VirtOffset {
        VirtOffset(offset)
    }
    pub fn offset(&self, pa: PhysAddr) -> VirtAddr {
        VirtAddr::init(pa.get() + self.0)
    }
}

/// A cluster-wide virtual address
#[derive(Copy, Clone)]
pub struct VirtAddr(usize);

impl VirtAddr {
    pub fn init(i: usize) -> VirtAddr {
        VirtAddr(i)
    }
    pub fn id_map(pa: PhysAddr, offset: VirtOffset) -> VirtAddr {
        offset.offset(pa)
    }
    pub fn forward(&self, step: usize) -> VirtAddr {
        VirtAddr(self.0 + step)
    }
    pub fn addr(&self) -> usize {
        self.0
    }
    pub fn offset(&self, offset: usize) -> VirtAddr {
        VirtAddr(self.0 + offset)
    }
    pub fn as_ptr(&self) -> *const () {
        self.0 as *const ()
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
            base: VirtAddr::id_map(range.base(), VirtOffset(0)),
            length: range.length(),
        }
    }

    pub fn step(self: &Self) -> VirtAddrRange {
        VirtAddrRange {
            base: self.base.forward(self.length),
            length: self.length,
        }
    }

    pub fn top(self: &Self) -> VirtAddr {
        VirtAddr(self.base.0 + self.length)
    }

    pub fn intersection(self: &Self, other: &Self) -> VirtAddrRange {
        use core::cmp::{max, min};

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
