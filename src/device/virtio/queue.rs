// SPDX-License-Identifier: Unlicense

//! A module for virtqueues.
//!
//!

use crate::device::RequestId;
use crate::pager::{
    Addr, AddrRange, OwnedMapping, Pager, Paging, PhysAddr, PhysAddrRange, VirtAddr,
};
use crate::{Error, Result};

use alloc::{sync::Arc, vec::Vec};

use core::fmt::Formatter;
use core::sync::atomic;
use core::sync::atomic::Ordering;
use core::{fmt, fmt::Debug};

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

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Descriptor {
    pub addr: PhysAddr,
    pub len: u32,
    pub id: RequestId,
    pub flags: DescriptorFlagsCopy,
}

impl Default for Descriptor {
    fn default() -> Self {
        Self {
            addr: PhysAddr::null(),
            len: 0,
            id: RequestId(!0),
            flags: DescriptorFlagsCopy::new(0),
        }
    }
}

impl Debug for Descriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Descriptor {{ addr: {:?}, len: {}, id: {:?}, flags: ",
            self.addr, self.len, self.id,
        )?;
        let mut first = false;
        if self.flags.is_set(DescriptorFlags::NEXT) {
            write!(f, "N")?;
            first = true;
        }
        if self.flags.is_set(DescriptorFlags::WRITE) {
            if first {
                write!(f, "|")?;
            }
            write!(f, "W")?;
            first = true;
        }
        if self.flags.is_set(DescriptorFlags::INDIRECT) {
            if first {
                write!(f, "|")?;
            }
            write!(f, "I")?;
            first = true;
        }
        if self.flags.is_set(DescriptorFlags::AVAIL) {
            if first {
                write!(f, "|")?;
            }
            write!(f, "A")?;
            first = true;
        }
        if self.flags.is_set(DescriptorFlags::USED) {
            if first {
                write!(f, "|")?;
            }
            write!(f, "U")?;
        }
        write!(f, "")
    }
}

impl Descriptor {
    pub fn new(addr_range: PhysAddrRange) -> Self {
        Self {
            addr: addr_range.base(),
            len: addr_range.length() as u32,
            ..Default::default()
        }
    }

    pub fn is_used(&self) -> bool {
        use DescriptorFlags::*;
        (self.flags.read(AVAIL) ^ self.flags.read(USED)) == 0
    }
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
    wrap_counter: bool,
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
        writeln!(f, "    wrap_counter: {}", self.wrap_counter)?;
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
            let virt_addr = virt_addr
                .increment(size_of::<EventSuppression>())
                .align_up(16);
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
                wrap_counter: true,
                avail_idx: 0,
                used_idx: 0,
            })
        }
    }

    pub fn index(&self) -> u32 {
        self.index
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

    pub fn next(&self, required: usize) -> Result<RequestId> {
        if self.descriptor_ring.len() as isize - (self.avail_idx as isize - self.used_idx as isize)
            < required as isize
        {
            return Err(Error::DeviceAtCapacity);
        }
        Ok(RequestId(self.avail_idx as u16))
    }

    pub fn submit(
        &mut self,
        id: RequestId,
        read_ranges: &Vec<(PhysAddr, u32)>,
        write_ranges: &Vec<(PhysAddr, u32)>,
    ) -> Result<()> {
        assert_eq!(id.0 as usize, self.avail_idx);
        let mut wrap_at_first = false;
        let mut last_idx: usize = !0;
        let read_len = read_ranges.len();
        for (i, (addr, len)) in read_ranges.iter().chain(write_ranges.iter()).enumerate() {
            let mut flags = DescriptorFlags::NEXT::SET.into();

            if i >= read_len {
                flags += DescriptorFlags::WRITE::SET;
            }

            if i == 0 {
                wrap_at_first = self.wrap_counter;
            } else {
                if self.wrap_counter {
                    flags += DescriptorFlags::AVAIL::SET;
                } else {
                    flags += DescriptorFlags::USED::SET;
                }
            }

            let mut desc = Descriptor {
                addr: *addr,
                len: *len,
                id,
                ..Default::default()
            };
            desc.flags.modify(flags);

            last_idx = i;
            self.descriptor_ring[i] = desc;

            self.avail_idx += 1;
            if self.avail_idx >= self.descriptor_ring.len() {
                self.avail_idx = 0;
                self.wrap_counter = !self.wrap_counter;
            }
        }
        self.descriptor_ring[last_idx]
            .flags
            .modify(DescriptorFlags::NEXT::CLEAR);
        atomic::fence(Ordering::SeqCst);
        self.descriptor_ring[id.0 as usize]
            .flags
            .modify(if wrap_at_first {
                DescriptorFlags::AVAIL::SET
            } else {
                DescriptorFlags::USED::SET
            });

        dbg!(&self.descriptor_ring[(id.0 as usize)..self.avail_idx]);
        dbg!(self.avail_idx);
        dbg!(self.wrap_counter);

        Ok(())
    }

    pub fn next_used(&self) -> Result<Descriptor> {
        info!("next_used: {}", self.used_idx);
        let used_desc = self.descriptor_ring[self.used_idx];
        if self.used_idx == self.avail_idx {
            Err(Error::DeviceIdle)
        } else if used_desc.is_used() {
            Ok(used_desc)
        } else {
            Err(Error::WouldBlock)
        }
    }

    pub fn consume_used(&mut self, span: usize) -> Result<()> {
        assert!(self.used_idx != self.avail_idx && self.descriptor_ring[self.used_idx].is_used());
        self.used_idx += span;
        if self.used_idx > self.descriptor_ring.len() {
            self.used_idx -= self.descriptor_ring.len();
        }
        Ok(())
    }
}
