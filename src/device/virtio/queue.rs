// SPDX-License-Identifier: Unlicense

//! A module for virtqueues.
//!
//!

use crate::pager::{Addr, OwnedMapping, Pager, Paging, PhysAddr, VirtAddr};
use crate::Result;

use core::fmt::Formatter;
use core::{fmt, fmt::Debug};

use alloc::sync::Arc;

use tock_registers::{register_bitfields, LocalRegisterCopy};

register_bitfields! [u16,
    DescriptorFlags [
        NEXT        0,
        WRITE       1,
        INDIRECT    2,
        AVAIL       7,
        USED        15,
    ]
];

type DescriptorFlagsCopy = LocalRegisterCopy<u16, DescriptorFlags::Register>;

#[repr(C)]
#[derive(Debug)]
pub struct Descriptor {
    addr: PhysAddr,
    len: u32,
    id: u16,
    flags: DescriptorFlagsCopy,
}

register_bitfields! [u16,
    EventSuppressionDesc [
        OFFSET OFFSET(0) NUMBITS(15),
        WRAP OFFSET(15) NUMBITS(1),
    ],
    EventSuppressionFlags [
        FLAGS OFFSET(0) NUMBITS(2) [
            Enable  = 0,
            Disable = 1,
            Desc    = 2,
        ]
    ]
];

type EventSuppressionDescCopy = LocalRegisterCopy<u16, EventSuppressionDesc::Register>;
type EventSuppressionFlagsCopy = LocalRegisterCopy<u16, EventSuppressionFlags::Register>;

#[repr(C)]
#[derive(Debug)]
struct EventSuppression {
    desc: EventSuppressionDescCopy,
    flags: EventSuppressionFlagsCopy,
}

pub struct PackedRing<'a> {
    index: u32,
    dma_range: Arc<OwnedMapping>,
    driver_event_suppression: &'a mut EventSuppression,
    device_event_suppression: &'a mut EventSuppression,
    descriptor_ring: &'a mut [Descriptor],
    wrap_counter: u8,
    avail_idx: usize,
    used_idx: usize,
}

impl Debug for PackedRing<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "PackedRing {{")?;
        writeln!(f, "    index: {}", self.index)?;
        writeln!(f, "    dma_range: {:?}", self.dma_range)?;
        writeln!(
            f,
            "    driver_event_suppression@{:?}: {:?}",
            self.driver_event_suppression as *const EventSuppression, self.driver_event_suppression,
        )?;
        writeln!(
            f,
            "    device_event_suppression@{:?}: {:?}",
            self.device_event_suppression as *const EventSuppression, self.device_event_suppression,
        )?;
        writeln!(
            f,
            "    descriptor_ring@0x{:?}/{}: {:?}",
            &self.descriptor_ring[0] as *const Descriptor,
            self.descriptor_ring.len(),
            self.descriptor_ring[0],
        )?;
        writeln!(f, "    wrap_counter: {:0b}", self.wrap_counter)?;
        writeln!(f, "    avail_idx: {}", self.avail_idx)?;
        writeln!(f, "    used_idx: {}", self.used_idx)?;
        write!(f, "}}")
    }
}

impl<'a> PackedRing<'a> {
    pub(in crate::device::virtio) fn new(index: u32) -> Result<Self> {
        use core::mem::size_of;

        unsafe {
            let dma_range = Pager::map_dma(1)?;
            let virt_addr = dma_range.base();
            let driver_event_suppression: &'a mut EventSuppression = virt_addr.as_mut_ref();
            let virt_addr = virt_addr.increment(size_of::<EventSuppression>());
            let device_event_suppression: &'a mut EventSuppression = virt_addr.as_mut_ref();
            let virt_addr = virt_addr.increment(size_of::<EventSuppression>());
            let ring_space_left = dma_range.length() - virt_addr.offset_above(dma_range.base());
            let descriptor_ring = core::slice::from_raw_parts_mut(
                virt_addr.into(),
                ring_space_left / core::mem::size_of::<Descriptor>(),
            );
            Ok(Self {
                index,
                dma_range,
                driver_event_suppression,
                device_event_suppression,
                descriptor_ring,
                wrap_counter: 0,
                avail_idx: 0,
                used_idx: 0,
            })
        }
    }

    pub fn descriptor_ring_len(&self) -> usize {
        self.descriptor_ring.len()
    }

    pub fn resize_descriptor_ring(&mut self, queue_size: u32) -> Result<()> {
        assert_le!(queue_size as usize, self.descriptor_ring.len());
        self.descriptor_ring = unsafe {
            core::slice::from_raw_parts_mut(
                &mut self.descriptor_ring[0] as *mut Descriptor,
                queue_size as usize,
            )
        };
        Ok(())
    }

    pub fn descriptor_ring_addr(&self) -> Result<PhysAddr> {
        let virt_addr = VirtAddr::from(&self.descriptor_ring[0] as &Descriptor);
        Pager::maps_to(virt_addr)
    }

    pub fn driver_event_suppression_addr(&self) -> Result<PhysAddr> {
        let virt_addr = VirtAddr::from(self.driver_event_suppression as &EventSuppression);
        Pager::maps_to(virt_addr)
    }

    pub fn device_event_suppression_addr(&self) -> Result<PhysAddr> {
        let virt_addr = VirtAddr::from(self.device_event_suppression as &EventSuppression);
        Pager::maps_to(virt_addr)
    }
}
