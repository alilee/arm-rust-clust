// SPDX-License-Identifier: Unlicense

//! A module for virtio devices.

mod block;
mod queue;

use super::make_addr_range;

use crate::pager::{
    Addr, AddrRange, FixedOffset, OwnedMapping, Pager, Paging, PhysAddr, PhysAddrRange, Translate,
    PAGESIZE_BYTES,
};
use crate::util::locked::Locked;
use crate::{Error, Result};

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

use dtb::StructItem;

use alloc::sync::Arc;
use core::sync::atomic;

register_bitfields! [u32,
    MagicValue [
        MAGIC OFFSET(0) NUMBITS(32) [
            Magic = 0x74726976,
        ]
    ],
    DeviceID [
        TYPE OFFSET(0) NUMBITS(32) [
            Invalid = 0x00,
            NetworkCard = 0x01,
            Block = 0x02,
            Console = 0x03,
            GPU = 0x10,
            Input = 0x12,
        ]
    ],
    Status [
        ACKNOWLEDGE         0,
        DRIVER              1,
        DRIVER_OK           2,
        FEATURES_OK         3,
        DEVICE_NEEDS_RESET  6,
        FAILED              7,
    ],
    FeaturesSelect [
        SELECTION OFFSET(0) NUMBITS(32) [
            Lo32 = 0x0,
            Hi32 = 0x1,
        ]
    ]
];

register_structs! {
    VirtIODevice {
        (0x000 => magic_value: ReadOnly<u32, MagicValue::Register>),
        (0x004 => version: ReadOnly<u32>),
        (0x008 => device_id: ReadOnly<u32, DeviceID::Register>),
        (0x00c => vendor_id: ReadOnly<u32>),
        (0x010 => device_features: ReadOnly<u32>),
        (0x014 => device_features_sel: WriteOnly<u32, FeaturesSelect::Register>),
        (0x018 => _reserved0),
        (0x020 => driver_features: WriteOnly<u32>),
        (0x024 => driver_features_sel: WriteOnly<u32, FeaturesSelect::Register>),
        (0x028 => _reserved1),
        (0x030 => queue_sel: WriteOnly<u32>),
        (0x034 => queue_num_max: ReadOnly<u32>),
        (0x038 => queue_num: WriteOnly<u32>),
        (0x03c => _reserved2),
        (0x044 => queue_ready: ReadWrite<u32>),
        (0x048 => _reserved3),
        (0x050 => queue_notify: WriteOnly<u32>),
        (0x054 => _reserved4),
        (0x060 => interrupt_status: ReadOnly<u32>),
        (0x064 => interrupt_ack: WriteOnly<u32>),
        (0x068 => _reserved5),
        (0x070 => status: ReadWrite<u32, Status::Register>),
        (0x074 => _reserved6),
        (0x080 => queue_desc_low: WriteOnly<u32>),
        (0x084 => queue_desc_high: WriteOnly<u32>),
        (0x088 => _reserved7),
        (0x090 => queue_driver_low: WriteOnly<u32>),
        (0x094 => queue_driver_high: WriteOnly<u32>),
        (0x098 => _reserved8),
        (0x0a0 => queue_device_low: WriteOnly<u32>),
        (0x0a4 => queue_device_high: WriteOnly<u32>),
        (0x0a8 => _reserved9),
        (0x0fc => config_generation: ReadOnly<u32>),
        (0x100 => @END),
    }
}

impl VirtIODevice {
    pub fn add_queue(&mut self, queue: &mut queue::PackedRing) -> Result<()> {
        self.queue_sel.set(0);
        atomic::fence(atomic::Ordering::SeqCst);

        if self.queue_ready.get() != 0 {
            return Err(Error::DeviceIncompatible);
        }

        let maximum_queue_size = self.queue_num_max.get();
        if maximum_queue_size == 0 {
            return Err(Error::DeviceIncompatible);
        }

        let queue_size = core::cmp::min(maximum_queue_size, queue.descriptor_ring_len() as u32);
        if queue_size < queue.descriptor_ring_len() as u32 {
            queue.resize_descriptor_ring(queue_size)?;
        }
        self.queue_num.set(queue_size);

        let (hi32, lo32) = queue.descriptor_ring_addr()?.hilo();
        self.queue_desc_low.set(lo32);
        self.queue_desc_high.set(hi32);

        let (hi32, lo32) = queue.driver_event_suppression_addr()?.hilo();
        self.queue_driver_low.set(lo32);
        self.queue_driver_high.set(hi32);

        let (hi32, lo32) = queue.device_event_suppression_addr()?.hilo();
        self.queue_device_low.set(lo32);
        self.queue_device_high.set(hi32);

        self.queue_ready.set(1);

        Ok(())
    }
}

fn make_intr(prop: StructItem) -> Result<(u32, u32, u32)> {
    let mut buf = [0u8; 32];
    let list = prop
        .value_u32_list(&mut buf)
        .or(Err(Error::DeviceIncompatible))?;
    assert_eq!(3, list.len());
    Ok((list[0], list[1], list[2]))
}

/// Initialise virtio devices.
pub fn init(dtb_root: dtb::StructItems) -> Result<()> {
    use alloc::collections::BTreeMap;
    {
        let mut buf = [0u8; 32];
        let (prop, _) = dtb_root.path_struct_items("/#size-cells").next().unwrap();
        let size_cells = prop
            .value_u32_list(&mut buf)
            .or(Err(Error::DeviceIncompatible))?[0];
        assert_eq!(2, size_cells);

        let (prop, _) = dtb_root
            .path_struct_items("/#address-cells")
            .next()
            .unwrap();
        let address_cells = prop
            .value_u32_list(&mut buf)
            .or(Err(Error::DeviceIncompatible))?[0];
        assert_eq!(2, address_cells);
    }

    let mut device_pages: BTreeMap<PhysAddr, (Arc<OwnedMapping>, FixedOffset)> = BTreeMap::new();

    for (node, node_iter) in dtb_root.path_struct_items("/virtio_mmio") {
        let (prop, _) = node_iter.clone().path_struct_items("reg").next().unwrap();
        let phys_addr_range = make_addr_range(prop)?;
        let page_base = phys_addr_range.base().align_down(PAGESIZE_BYTES);

        if !device_pages.contains_key(&page_base) {
            let mapping = Pager::map_device(PhysAddrRange::page_at(page_base))?;
            let translation = FixedOffset::new(page_base, mapping.base());
            device_pages.insert(page_base, (mapping, translation));
        };

        let (mapping, translation) = device_pages.get(&page_base).expect("device_pages.get");
        info!("translation: {:?}", translation);
        let reg = translation.translate_phys(phys_addr_range.base())?;
        let device: &mut VirtIODevice = unsafe { reg.as_mut_ref() };

        if device.magic_value.read(MagicValue::MAGIC) == MagicValue::MAGIC::Magic.value
            && device.version.get() == 2
        {
            match device.device_id.read_as_enum(DeviceID::TYPE) {
                Some(DeviceID::TYPE::Value::Invalid) => {}
                Some(DeviceID::TYPE::Value::Block) => {
                    debug!(
                        "version 2 block device ({:?}) found at {:?}",
                        device.device_id.read(DeviceID::TYPE),
                        node
                    );
                    let (interrupt, _) = node_iter
                        .clone()
                        .path_struct_items("interrupts")
                        .next()
                        .unwrap();
                    let interrupt = make_intr(interrupt)?;
                    let block_device = block::init(
                        node.name().or(Err(Error::UnexpectedValue))?,
                        mapping.clone(),
                        reg,
                        interrupt.1,
                    )?;
                    super::BLOCK_DEVICES
                        .lock()
                        .insert(block_device.name(), Locked::new(block_device));
                }
                Some(_) => {
                    info!(
                        "unimplemented DeviceID::TYPE.value {:?} ({:?})",
                        device.device_id.read(DeviceID::TYPE),
                        reg
                    );
                }
                None => {
                    error!(
                        "Unknown DeviceID::TYPE {:?}",
                        device.device_id.read(DeviceID::TYPE)
                    );
                }
            };
        }
    }

    Ok(())
}
