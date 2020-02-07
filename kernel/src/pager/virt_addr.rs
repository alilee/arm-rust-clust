use crate::pager::{Page, PhysAddr, PhysAddrRange, PAGESIZE_BYTES};

use core::fmt::{Debug, Error, Formatter};

/// A cluster-wide virtual address
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct VirtAddr(usize);

impl VirtAddr {
    pub fn new(addr: usize) -> VirtAddr {
        VirtAddr(addr)
    }
    pub const fn new_const(addr: usize) -> VirtAddr {
        VirtAddr(addr)
    }
    pub fn id_map(phys_addr: PhysAddr) -> VirtAddr {
        unsafe { VirtAddr::new(phys_addr.get()) }
    }
    pub unsafe fn get(&self) -> usize {
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

    //    pub fn target_map(phys_range: PhysAddrRange, virt_base: VirtAddr) -> (Self, PhysOffset) {
    //        let virt_range = Self {
    //            base: virt_base,
    //            length: phys_range.length(),
    //        };
    //        let phys_offset = PhysOffset::new(virt_base, phys_range.base());
    //        (virt_range, phys_offset)
    //    }

    pub fn trim_left_pages(&self, pages: usize) -> VirtAddrRange {
        let length = pages * PAGESIZE_BYTES;
        Self {
            base: unsafe { self.base.increment(length) },
            length: self.length - length,
        }
    }

    pub fn step(self: &Self) -> Self {
        Self {
            base: unsafe { self.base.increment(self.length) },
            length: self.length,
        }
    }

    pub fn increment(&self, incr: usize) -> Self {
        Self {
            base: unsafe { self.base.increment(incr) },
            length: self.length,
        }
    }

    pub const fn top(self: &Self) -> VirtAddr {
        VirtAddr(self.base.0 + self.length)
    }

    pub const fn length_in_pages(&self) -> usize {
        self.length / PAGESIZE_BYTES
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
pub struct AddrOffsetDown {
    delta: usize,
}

impl AddrOffsetDown {
    pub fn new_phys_offset(high_addr: PhysAddr, low_addr: PhysAddr) -> Self {
        assert!(high_addr > low_addr);
        let delta = unsafe { high_addr.get() - low_addr.get() };
        Self { delta }
    }

    pub fn reverse_offset_virt_addr(&self, virt_base: VirtAddr) -> VirtAddr {
        unsafe { virt_base.increment(self.delta) }
    }
}

impl Debug for AddrOffsetDown {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "- {:#x}", self.delta)
    }
}

#[derive(Copy, Clone)]
pub struct AddrOffsetUp {
    delta: usize,
}

impl AddrOffsetUp {
    pub fn id_map() -> Self {
        Self { delta: 0 }
    }

    pub fn reverse_translation(phys_addr: PhysAddr, virt_base: VirtAddr) -> Self {
        assert!(virt_base > VirtAddr::id_map(phys_addr));
        Self {
            delta: unsafe { virt_base.get() - phys_addr.get() },
        }
    }
    pub fn reverse_translate_range(&self, phys_range: PhysAddrRange) -> VirtAddrRange {
        VirtAddrRange::id_map(phys_range).increment(self.delta)
    }

    pub fn reverse_translate_fn(&self, addr: fn() -> !) -> fn() -> ! {
        use core::mem;
        let va = VirtAddr::from(addr as *const ());
        let va = unsafe { va.increment(self.delta) };
        unsafe { mem::transmute::<*const (), fn() -> !>(va.as_ptr()) }
    }

    pub fn translate(&self, virt_addr: VirtAddr) -> PhysAddr {
        unsafe {
            let addr = virt_addr.get();
            PhysAddr::new(addr + self.delta)
        }
    }

    pub unsafe fn get_offset(&self) -> usize {
        self.delta
    }
}

impl Debug for AddrOffsetUp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "+ {:#x}", self.delta)
    }
}
