// SPDX-License-Identifier: Unlicense

use crate::archs::{arch::Arch, PagerTrait};
use crate::pager::FixedOffset;
use crate::{Error, Result};

use super::{Addr, AddrRange, AttributeField, Attributes, PhysAddrRange, VirtAddr, VirtAddrRange};

use core::fmt;
use core::fmt::{Debug, Formatter};

/// Content within Range.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RangeContent {
    RAM,
    KernelText,
    KernelStatic,
    KernelData,
    FrameTable,
    KernelStack,
    KernelHeap,
    Device,
}

/// Range requiring to be mapped.
#[derive(Copy, Clone)]
struct KernelExtent {
    content: RangeContent,
    virt_range_align: usize,
    virt_range_min_extent: usize,
    virt_range_gap: &'static dyn Fn() -> Option<usize>,
    phys_addr_range: &'static dyn Fn() -> Option<PhysAddrRange>,
    attributes: Attributes,
    virt_range: Option<VirtAddrRange>,
}

impl Debug for KernelExtent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "KernelExtent {{{:?} => {:?}, {:#x}, {:#x}, {:?}, {:?}}}",
            self.virt_range,
            self.content,
            self.virt_range_align,
            self.virt_range_min_extent,
            (self.phys_addr_range)(),
            self.attributes
        )
    }
}

static mut FRAME_TABLE_RANGE: Option<PhysAddrRange> = None;

const GB: usize = 1024 * 1024 * 1024;

static mut LAYOUT: [KernelExtent; 8] = [
    KernelExtent {
        content: RangeContent::RAM,
        virt_range_align: 1 * GB,
        virt_range_min_extent: 8 * GB,
        virt_range_gap: &{ || None },
        phys_addr_range: &{ || Some(Arch::ram_range().expect("Arch::ram_range")) },
        attributes: Attributes::RAM,
        virt_range: None,
    },
    KernelExtent {
        content: RangeContent::KernelText,
        virt_range_align: 1 * GB,
        virt_range_min_extent: 0,
        virt_range_gap: &{
            || {
                Some(
                    Arch::text_image()
                        .base()
                        .offset_above(Arch::ram_range().expect("Arch::ram_range").base()),
                )
            }
        },
        phys_addr_range: &{ || Some(Arch::text_image()) },
        attributes: Attributes::KERNEL_EXEC,
        virt_range: None,
    },
    KernelExtent {
        content: RangeContent::KernelStatic,
        virt_range_align: 0,
        virt_range_min_extent: 0,
        virt_range_gap: &{ || None },
        phys_addr_range: &{ || Some(Arch::static_image()) },
        attributes: Attributes::KERNEL_STATIC,
        virt_range: None,
    },
    KernelExtent {
        content: RangeContent::KernelData,
        virt_range_align: 0,
        virt_range_min_extent: 0,
        virt_range_gap: &{ || None },
        phys_addr_range: &{ || Some(Arch::data_image()) },
        attributes: Attributes::KERNEL_DATA,
        virt_range: None,
    },
    KernelExtent {
        content: RangeContent::FrameTable,
        virt_range_align: 1 * GB,
        virt_range_min_extent: 1 * GB,
        virt_range_gap: &{ || None },
        phys_addr_range: &{ || unsafe { FRAME_TABLE_RANGE } },
        attributes: Attributes::KERNEL_DATA.set(AttributeField::Block),
        virt_range: None,
    },
    KernelExtent {
        content: RangeContent::KernelStack,
        virt_range_align: 1 * GB,
        virt_range_min_extent: 1 * GB,
        virt_range_gap: &{ || None },
        phys_addr_range: &{ || None },
        attributes: Attributes::KERNEL_DATA,
        virt_range: None,
    },
    KernelExtent {
        content: RangeContent::KernelHeap,
        virt_range_align: 1 * GB,
        virt_range_min_extent: 8 * GB,
        virt_range_gap: &{ || None },
        phys_addr_range: &{ || None },
        attributes: Attributes::KERNEL_DATA,
        virt_range: None,
    },
    KernelExtent {
        content: RangeContent::Device,
        virt_range_align: 1 * GB,
        virt_range_min_extent: 1 * GB,
        virt_range_gap: &{ || None },
        phys_addr_range: &{ || None },
        attributes: Attributes::DEVICE,
        virt_range: None,
    },
];

static mut MEM_FIXED_OFFSET: FixedOffset = FixedOffset::identity();

/// Initialise
pub fn init(frame_table_range: PhysAddrRange) -> Result<()> {
    info!("init");
    info!("Kernel base: {:?}", Arch::kernel_base());

    unsafe {
        FRAME_TABLE_RANGE = Some(frame_table_range);
    }

    let mut virt_addr = Arch::kernel_base();
    unsafe {
        for extent in LAYOUT.iter_mut() {
            let base = if extent.virt_range_align > 0 {
                virt_addr.align_up(extent.virt_range_align)
            } else {
                virt_addr
            };
            let virt_range_gap = (extent.virt_range_gap)();
            let base = match virt_range_gap {
                Some(gap) => base.increment(gap),
                None => base,
            };

            let phys_addr_range = (extent.phys_addr_range)();

            let length = if let Some(phys_addr_range) = phys_addr_range {
                core::cmp::max(phys_addr_range.length(), extent.virt_range_min_extent)
            } else {
                extent.virt_range_min_extent
            };
            extent.virt_range = Some(VirtAddrRange::new(base, length));

            if extent.content == RangeContent::RAM {
                let phys_addr = phys_addr_range.unwrap().base();
                let translation = FixedOffset::new(phys_addr, base);
                MEM_FIXED_OFFSET = translation;
            }

            virt_addr = extent.virt_range.unwrap().step().base();
        }
    }

    Ok(())
}

/// Range requiring to be mapped.
#[derive(Debug)]
pub struct KernelRange {
    pub content: RangeContent,
    pub virt_addr_range: VirtAddrRange,
    pub phys_addr_range: Option<PhysAddrRange>,
    pub attributes: Attributes,
}

impl KernelRange {
    fn from(extent: &KernelExtent) -> Self {
        Self {
            content: extent.content,
            virt_addr_range: extent.virt_range.expect("Layout not initialised"),
            phys_addr_range: (extent.phys_addr_range)(),
            attributes: extent.attributes,
        }
    }
}

/// Iterable over the memory layout.
pub struct LayoutIterator {
    i: usize,
    next_base: VirtAddr,
}

impl Debug for LayoutIterator {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), fmt::Error> {
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
        unsafe {
            if self.i >= LAYOUT.len() {
                return None;
            }
            info!("{:?}", &LAYOUT[self.i]);
            let result = KernelRange::from(&LAYOUT[self.i]);
            self.next_base = result.virt_addr_range.step().base();
            self.i += 1;
            Some(result)
        }
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

/// Get the offset of real RAM from the kernel-mapped area.
#[inline(always)]
pub fn mem_translation() -> &'static impl super::Translate {
    unsafe { &MEM_FIXED_OFFSET }
}

/// Search the kernel layout for the virtual address range of the first kernel heap area.
pub fn get_range(content: RangeContent) -> Result<VirtAddrRange> {
    unsafe {
        LAYOUT
            .iter()
            .find(|extent| (*extent).content == content)
            .and_then(|e| e.virt_range)
            .ok_or(Error::UnInitialised)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pager::PhysAddr;

    #[test]
    fn calculate() {
        assert_err!(get_range(RangeContent::KernelHeap));
        init(PhysAddrRange::new(PhysAddr::at(0x4000_0000), 0x1000)).unwrap();
        for item in layout().unwrap() {
            assert_eq!(item.virt_addr_range, get_range(item.content).unwrap());
        }
    }
}
