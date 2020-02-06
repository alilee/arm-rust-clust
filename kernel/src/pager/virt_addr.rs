use crate::pager::{Page, PhysAddr, PhysAddrRange, PAGESIZE_BYTES};

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
pub struct PhysVirtTranslation(usize);

impl PhysVirtTranslation {
    pub unsafe fn between(pa: PhysAddr, va: VirtAddr) -> Self {
        assert!(pa.get() < va.get());
        Self(va.0 - pa.get())
    }
    pub unsafe fn translate(&self, pa: PhysAddr) -> VirtAddr {
        VirtAddr::new(pa.get() + self.0)
    }
    pub unsafe fn xget(&self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone)]
pub struct XVirtOffset(usize);

impl XVirtOffset {
    pub fn between(low_va: VirtAddr, high_va: VirtAddr) -> Self {
        assert!(low_va < high_va);
        Self(high_va.0 - low_va.0)
    }
    pub fn offset(&self, virt_addr: VirtAddr) -> VirtAddr {
        unsafe { virt_addr.increment(self.0) }
    }
    pub fn offset_fn(&self, addr: fn() -> !) -> fn() -> ! {
        use core::mem;
        let va = VirtAddr::from(addr as *const ());
        let va = self.offset(va);
        unsafe { mem::transmute::<*const (), fn() -> !>(va.as_ptr()) }
    }
    pub unsafe fn get(&self) -> usize {
        self.0
    }
}

impl Debug for XVirtOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "+0x{:08x}", self.0)
    }
}

/// A cluster-wide virtual address
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct VirtAddr(usize);

impl VirtAddr {
    pub fn new(addr: usize) -> VirtAddr {
        VirtAddr(addr)
    }
    pub fn id_map(pa: PhysAddr) -> VirtAddr {
        unsafe { VirtAddr::new(pa.get()) }
    }
    pub unsafe fn addr(&self) -> usize {
        self.0
    }
    pub unsafe fn increment(&self, incr: usize) -> VirtAddr {
        VirtAddr(self.0 + incr)
    }
    pub unsafe fn decrement(&self, decr: usize) -> VirtAddr {
        VirtAddr(self.0 - decr)
    }

    pub unsafe fn as_ptr(&self) -> *const () {
        self.0 as *const ()
    }
    pub unsafe fn as_mut_ptr(&self) -> *mut () {
        self.0 as *mut ()
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "VirtAddr(0x{:016x})", self.0)
    }
}

impl From<*const ()> for VirtAddr {
    fn from(p: *const ()) -> Self {
        Self::new(p as usize)
    }
}

impl From<*const Page> for VirtAddr {
    fn from(p: *const Page) -> Self {
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
    pub fn new(base: VirtAddr, length: usize) -> Self {
        Self { base, length }
    }

    pub fn id_map(range: PhysAddrRange) -> Self {
        Self {
            base: VirtAddr::id_map(range.base()),
            length: range.length(),
        }
    }

    pub fn target_map(phys_range: PhysAddrRange, virt_base: VirtAddr) -> (Self, PhysOffset) {
        let virt_range = Self {
            base: virt_base,
            length: phys_range.length(),
        };
        let phys_offset = PhysOffset::new(virt_base, phys_range.base());
        (virt_range, phys_offset)
    }

    pub fn trim_left_pages(&self, pages: usize) -> VirtAddrRange {
        let length = pages * PAGESIZE_BYTES;
        Self {
            base: unsafe { self.base.increment(length) },
            length: self.length - length,
        }
    }

    pub fn step(self: &Self) -> VirtAddrRange {
        VirtAddrRange {
            base: unsafe { self.base.increment(self.length) },
            length: self.length,
        }
    }

    pub const fn top(self: &Self) -> VirtAddr {
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

    pub fn covers(&self, other: &Self) -> bool {
        self.base <= other.base && self.top() >= other.top()
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
