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
    pub fn new(offset: usize) -> VirtOffset {
        VirtOffset(offset)
    }
    pub const fn new_const(offset: usize) -> VirtOffset {
        VirtOffset(offset)
    }
    pub fn offset(&self, pa: PhysAddr) -> VirtAddr {
        VirtAddr::new(pa.get() + self.0)
    }
}

impl Debug for VirtOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "+0x{:08x}", self.0)
    }
}

/// A cluster-wide virtual address
#[derive(Copy, Clone)]
pub struct VirtAddr(usize);

impl VirtAddr {
    pub fn new(addr: usize) -> VirtAddr {
        VirtAddr(addr)
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
    pub fn increment(&self, incr: usize) -> VirtAddr {
        VirtAddr(self.0 + incr)
    }
    pub fn offset(&self, offset: VirtOffset) -> VirtAddr {
        VirtAddr(self.0 + offset.0)
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

impl From<*const ()> for VirtAddr {
    fn from(p: *const ()) -> Self {
        Self::new(p as usize)
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

    pub fn target_map(
        phys_range: PhysAddrRange,
        virt_base: VirtAddr,
    ) -> (VirtAddrRange, PhysOffset) {
        let virt_range = VirtAddrRange {
            base: virt_base,
            length: phys_range.length(),
        };
        let phys_offset = PhysOffset::new(virt_base, phys_range.base());
        (virt_range, phys_offset)
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
    pub fn base(&self) -> VirtAddr {
        self.base
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

#[derive(Copy, Clone)]
pub struct PhysOffset {
    delta: isize,
}

impl PhysOffset {
    pub fn id_map() -> Self {
        Self { delta: 0 }
    }
    pub fn new(va: VirtAddr, pa: PhysAddr) -> Self {
        let delta = (pa.get() as isize) - (va.0 as isize);
        Self { delta }
    }
    pub fn translate(&self, va: VirtAddr) -> PhysAddr {
        PhysAddr::new((va.addr() as isize + self.delta) as usize)
    }
}

impl Debug for PhysOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "VA {:#x} PA", self.delta)
    }
}
