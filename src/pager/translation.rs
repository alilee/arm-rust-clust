// SPDX-License-Identifier: Unlicense

//! Translation from physical to virtual addresses, based on policies..
//!
//! At different times during the boot sequence, accessible memory may be mapped
//! at the same or different virtual addresses.

use super::PhysAddr;
use super::VirtAddr;

use core::fmt::{Debug, Formatter};

/// Able to translate
pub trait Translate {
    /// Translate a physical address to a virtual address
    fn translate(&self, phys_addr: PhysAddr) -> VirtAddr;
}

/// Translation such that physical address is same as virtual address
#[derive(Copy, Clone)]
pub struct Identity;

impl Identity {
    /// Construct a new identity translation
    pub fn new() -> Self {
        Identity {}
    }
}

impl Translate for Identity {
    /// Calculate the virtual address offset from the given physical page.
    fn translate(&self, phys_addr: PhysAddr) -> VirtAddr {
        unsafe { VirtAddr::identity_mapped(phys_addr) }
    }
}

/// A policy defining the translation using a fixed offset.
#[derive(Copy, Clone)]
pub struct FixedOffset(usize);

impl Debug for FixedOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "FixedOffset(+0x{:08x})", self.0)
    }
}

impl FixedOffset {
    /// Define translation as different between reference physical and virtual addresses.
    ///
    /// NOTE: pa must not be above va.
    pub fn new(phys_addr: PhysAddr, virt_addr: VirtAddr) -> Self {
        unsafe {
            let nominal_virt_addr = VirtAddr::identity_mapped(phys_addr);
            assert!(nominal_virt_addr <= virt_addr);
            Self(nominal_virt_addr.increment_to(virt_addr))
        }
    }
}

impl Translate for FixedOffset {
    /// Calculate the virtual address offset from the given physical page.
    fn translate(&self, phys_addr: PhysAddr) -> VirtAddr {
        let virt_addr = unsafe { VirtAddr::identity_mapped(phys_addr) };
        virt_addr.increment(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let pa = PhysAddr::at(0x4800_0000);
        unsafe {
            assert_eq!(VirtAddr::identity_mapped(pa), Identity::new().translate(pa));
        }
    }

    #[test]
    fn offset() {
        let pa = PhysAddr::at(0x4800_0000);
        let va = VirtAddr::at(0x1_4800_0000);
        let fixed = FixedOffset::new(pa, va);
        assert_eq!(va, fixed.translate(pa));

        let pa2 = PhysAddr::at(0x5800_0000);
        assert_eq!(VirtAddr::at(0x1_5800_0000), fixed.translate(pa2));
    }
}
