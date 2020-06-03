// SPDX-License-Identifier: Unlicense

//! Translation from virtual to physical addresses, linear or based on policies.
//!
//! At different times during the boot sequence, accessible memory may be mapped
//! at the same or different virtual addresses.

use super::{Addr, PhysAddr, VirtAddr};

use crate::{Error, Result};

use core::fmt::{Debug, Formatter};

/// Able to translate.
pub trait Translate {
    /// Translate a virtual address to a physical address.
    fn translate(&self, virt_addr: VirtAddr) -> PhysAddr;

    /// Translate a virtual address to an option physical address.
    fn translate_maybe(&self, virt_addr: VirtAddr) -> Option<PhysAddr>;

    /// Reverse translate a physical address to a virtual address, if defined..
    fn translate_phys(&self, _phys_addr: PhysAddr) -> Result<VirtAddr> {
        Err(Error::Undefined)
    }
}

/// Translation such that physical address is always null.
#[derive(Copy, Clone)]
pub struct NullTranslation;

impl Debug for NullTranslation {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "NullTranslation()")
    }
}

impl NullTranslation {
    /// Construct a new null translation
    pub fn new() -> Self {
        Self {}
    }
}

impl Translate for NullTranslation {
    fn translate(&self, _virt_addr: VirtAddr) -> PhysAddr {
        PhysAddr::null()
    }

    fn translate_maybe(&self, _virt_addr: VirtAddr) -> Option<PhysAddr> {
        None
    }
}

/// Translation such that physical address is same as virtual address
#[derive(Copy, Clone)]
pub struct Identity;

impl Identity {
    /// Construct a new identity translation
    pub fn new() -> Self {
        Self {}
    }
}

impl Debug for Identity {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Identity()")
    }
}

impl Translate for Identity {
    fn translate(&self, virt_addr: VirtAddr) -> PhysAddr {
        PhysAddr::identity_mapped(virt_addr)
    }
    fn translate_maybe(&self, virt_addr: VirtAddr) -> Option<PhysAddr> {
        Some(self.translate(virt_addr))
    }
    fn translate_phys(&self, phys_addr: PhysAddr) -> Result<VirtAddr> {
        unsafe { Ok(VirtAddr::identity_mapped(phys_addr)) }
    }
}

/// A policy defining the translation using a fixed offset.
///
/// NOTE: This translates downward from higher virtual addresses to lower physical addresses
/// such as you would need kernel is at the top of the virtual address space..
#[derive(Copy, Clone)]
pub struct FixedOffset(usize);

impl Debug for FixedOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "FixedOffset(-0x{:08x})", self.0)
    }
}

impl From<Identity> for FixedOffset {
    fn from(_: Identity) -> Self {
        Self(0)
    }
}

impl FixedOffset {
    /// Define translation as difference between reference physical and virtual addresses.
    ///
    /// NOTE: pa must not be above va.
    pub fn new(phys_addr: PhysAddr, virt_addr: VirtAddr) -> Self {
        unsafe {
            let nominal_phys_addr = VirtAddr::identity_mapped(phys_addr);
            assert!(nominal_phys_addr <= virt_addr);
            Self(virt_addr.offset_above(nominal_phys_addr))
        }
    }

    /// Create null fixed offset - identity mapping.
    ///
    /// TODO: Remove use of FixedOffset where identity would be preferred.
    pub const fn identity() -> Self {
        Self(0)
    }
}

impl Translate for FixedOffset {
    /// Calculate the virtual address offset from the given physical page.
    fn translate(&self, virt_addr: VirtAddr) -> PhysAddr {
        PhysAddr::identity_mapped(virt_addr.decrement(self.0))
    }
    fn translate_maybe(&self, virt_addr: VirtAddr) -> Option<PhysAddr> {
        Some(self.translate(virt_addr))
    }
    fn translate_phys(&self, phys_addr: PhysAddr) -> Result<VirtAddr> {
        unsafe { Ok(VirtAddr::identity_mapped(phys_addr).increment(self.0)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null() {
        let virt_addr = VirtAddr::at(0x4800_0000);
        assert_eq!(
            PhysAddr::null(),
            NullTranslation::new().translate(virt_addr)
        );
    }

    #[test]
    fn identity() {
        let virt_addr = VirtAddr::at(0x4800_0000);
        assert_eq!(
            PhysAddr::identity_mapped(virt_addr),
            Identity::new().translate(virt_addr)
        );
    }

    #[test]
    fn offset() {
        let phys_addr = PhysAddr::at(0x4800_0000);
        let virt_addr = VirtAddr::at(0x1_4800_0000);
        let fixed = FixedOffset::new(phys_addr, virt_addr);
        assert_eq!(phys_addr, fixed.translate(virt_addr));

        let virt_addr = VirtAddr::at(0x1_5800_0000);
        assert_eq!(PhysAddr::at(0x5800_0000), fixed.translate(virt_addr));
    }
}
