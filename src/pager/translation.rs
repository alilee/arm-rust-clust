// SPDX-License-Identifier: Unlicense

//! Translation from physical to virtual addresses, based on policies..
//!
//! At different times during the boot sequence, accessible memory may be mapped
//! at the same or different virtual addresses.

use super::PhysAddr;
use super::VirtAddr;

/// Able to translate
pub trait Translate {
    /// Translate a physical address to a virtual address
    fn translate(&self, phys_addr: PhysAddr) -> VirtAddr;
}

/// Translation such that physical address is same as virtual address
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
#[derive(Copy, Clone, Debug)]
pub struct FixedOffset(usize);

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
        let pa = PhysAddr(0x4800_0000);
        assert_eq!(pa.identity_map(), Identity::new().translate(pa));
    }

    #[test]
    fn offset() {
        let pa = PhysAddr(0x4800_0000);
        let va = VirtAddr(0x1_4800_0000);
        let fixed = FixedOffset::new(pa, va);
        assert_eq!(va, fixed.translate(pa));

        let pa2 = PhysAddr(0x5800_0000);
        assert_eq!(VirtAddr(0x1_5800_0000), fixed.translate(pa2));
    }
}
