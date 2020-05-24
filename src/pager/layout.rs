// SPDX-License-Identifier: Unlicense

use super::{Attributes, FixedOffset, PhysAddrRange, VirtAddr, VirtAddrRange};
use crate::archs::{arch::Arch, ArchTrait};
use crate::Result;

use core::fmt::{Debug, Error, Formatter};

/// Initialise
pub fn init() -> Result<()> {
    info!("init");
    info!("Kernel base: {:?}", Arch::kernel_base());
    Ok(())
}

enum KernelExtent {
    RAM {
        phys_addr_range: &'static dyn Fn() -> PhysAddrRange,
        virt_addr_extent: usize,
        attributes: Attributes,
    },
    Image {
        phys_addr_range: &'static dyn Fn() -> PhysAddrRange,
        virt_addr_extent: usize,
        attributes: Attributes,
    },
    Device {
        virt_addr_extent: usize,
        attributes: Attributes,
    },
    L3PageTables {
        virt_addr_extent: usize,
        attributes: Attributes,
    },
    Heap {
        virt_addr_extent: usize,
        attributes: Attributes,
    },
}

impl Debug for KernelExtent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use KernelExtent::*;
        match self {
            RAM {
                phys_addr_range,
                virt_addr_extent,
                attributes,
            } => write!(
                f,
                "RAM {{ {:?}, extent: {} GB, {:?} }}",
                phys_addr_range(),
                virt_addr_extent / GB,
                attributes
            ),
            Image {
                phys_addr_range,
                virt_addr_extent,
                attributes,
            } => write!(
                f,
                "Image {{ {:?}, extent: {} GB, {:?} }}",
                phys_addr_range(),
                virt_addr_extent / GB,
                attributes
            ),
            Device {
                virt_addr_extent,
                attributes,
            } => write!(
                f,
                "Device {{ extent: {} GB, {:?} }}",
                virt_addr_extent / GB,
                attributes
            ),
            L3PageTables {
                virt_addr_extent,
                attributes,
            } => write!(
                f,
                "Device {{ extent: {} GB, {:?} }}",
                virt_addr_extent / GB,
                attributes
            ),
            Heap {
                virt_addr_extent,
                attributes,
            } => write!(
                f,
                "Heap {{ extent: {} GB, {:?} }}",
                virt_addr_extent / GB,
                attributes
            ),
        }
    }
}

impl KernelExtent {
    fn virt_addr_extent(&self) -> usize {
        use KernelExtent::*;
        match self {
            RAM {
                virt_addr_extent, ..
            } => *virt_addr_extent,
            Image {
                virt_addr_extent, ..
            } => *virt_addr_extent,
            Device {
                virt_addr_extent, ..
            } => *virt_addr_extent,
            L3PageTables {
                virt_addr_extent, ..
            } => *virt_addr_extent,
            Heap {
                virt_addr_extent, ..
            } => *virt_addr_extent,
        }
    }
}

const GB: usize = 1024 * 1024 * 1024;

const LAYOUT: [KernelExtent; 5] = [
    KernelExtent::RAM {
        phys_addr_range: &{ || Arch::ram_range().expect("Arch::ram_range") },
        virt_addr_extent: 4 * GB,
        attributes: Attributes::RAM,
    },
    KernelExtent::Image {
        phys_addr_range: &{ || PhysAddrRange::boot_image() },
        virt_addr_extent: 1 * GB,
        attributes: Attributes::KERNEL_EXEC,
    },
    KernelExtent::Device {
        virt_addr_extent: 1 * GB,
        attributes: Attributes::DEVICE,
    },
    KernelExtent::L3PageTables {
        virt_addr_extent: 8 * GB,
        attributes: Attributes::KERNEL_MEM,
    },
    KernelExtent::Heap {
        virt_addr_extent: 8 * GB,
        attributes: Attributes::KERNEL_MEM,
    },
];

/// Range requiring to be mapped
#[derive(Debug)]
pub enum KernelRange {
    RAM(VirtAddrRange, FixedOffset, Attributes),
    Image(VirtAddrRange, FixedOffset, Attributes),
    Device(VirtAddrRange, Attributes),
    L3PageTables(VirtAddrRange, Attributes),
    Heap(VirtAddrRange, Attributes),
}

impl KernelRange {
    fn from(virt_addr: VirtAddr, extent: &KernelExtent) -> Self {
        match extent {
            KernelExtent::RAM {
                phys_addr_range,
                attributes,
                ..
            } => KernelRange::RAM(
                VirtAddrRange::new(virt_addr, phys_addr_range().length()),
                FixedOffset::new(phys_addr_range().base(), virt_addr),
                *attributes,
            ),
            KernelExtent::Image {
                phys_addr_range,
                attributes,
                ..
            } => KernelRange::Image(
                VirtAddrRange::new(virt_addr, phys_addr_range().length()),
                FixedOffset::new(phys_addr_range().base(), virt_addr),
                *attributes,
            ),
            KernelExtent::Device {
                virt_addr_extent,
                attributes,
            } => KernelRange::Device(virt_addr.extend(*virt_addr_extent), *attributes),
            KernelExtent::L3PageTables {
                virt_addr_extent,
                attributes,
            } => KernelRange::L3PageTables(virt_addr.extend(*virt_addr_extent), *attributes),
            KernelExtent::Heap {
                virt_addr_extent,
                attributes,
            } => KernelRange::Heap(virt_addr.extend(*virt_addr_extent), *attributes),
        }
    }
}

/// Iterable over the memory layout.
pub struct LayoutIterator {
    i: usize,
    next_base: VirtAddr,
}

impl Debug for LayoutIterator {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), Error> {
        write!(f, "Layout")
    }
}

impl LayoutIterator {
    fn new() -> Self {
        Self {
            i: 0,
            next_base: Arch::kernel_base(),
        }
    }
}

impl Iterator for LayoutIterator {
    type Item = KernelRange;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= LAYOUT.len() {
            return None;
        }
        trace!("{:?}: {:?}", self.next_base, &LAYOUT[self.i]);
        let result = Some(KernelRange::from(self.next_base, &LAYOUT[self.i]));
        self.next_base = self.next_base.increment(LAYOUT[self.i].virt_addr_extent());
        self.i += 1;
        result
    }
}

pub struct Layout;

/// Iterate through the layout
pub fn layout() -> Result<Layout> {
    Ok(Layout {})
}

impl IntoIterator for Layout {
    type Item = KernelRange;
    type IntoIter = LayoutIterator;

    fn into_iter(self) -> Self::IntoIter {
        LayoutIterator::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate() {
        for item in layout().unwrap() {
            info!("{:?}", item);
        }
    }
}
