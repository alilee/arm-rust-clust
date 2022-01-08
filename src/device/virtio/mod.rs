// SPDX-License-Identifier: Unlicense

//! A module for virtio devices.

mod block;

use crate::pager::{
    Addr, AddrRange, FixedOffset, Pager, Paging, PhysAddr, PhysAddrRange, Translate, PAGESIZE_BYTES,
};
use crate::Result;

use tock_registers::interfaces::Readable;
use tock_registers::registers::{ReadOnly, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

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
    DeviceFeatures [
        /// Block features
        BLK_SIZE_MAX        1,
        BLK_SEG_MAX         2,
        BLK_GEOMETRY        4,
        BLK_RO              5,
        BLK_BLK_SIZE        6,
        BLK_FLUSH           9,
        BLK_TOPOLOGY        10,
        BLK_CONFIG_WCE      11,
        BLK_DISCARD         13,
        BLK_WRITE_ZEROES    14,
    ]
];

register_structs! {
    VirtIODevice {
        (0x000 => magic_value: ReadOnly<u32, MagicValue::Register>),
        (0x004 => version: ReadOnly<u32>),
        (0x008 => device_id: ReadOnly<u32, DeviceID::Register>),
        (0x00c => vendor_id: ReadOnly<u32>),
        (0x010 => device_features: ReadOnly<u32, DeviceFeatures::Register>),
        (0x014 => device_features_sel: WriteOnly<u32>),
        (0x018 => _reserved),
        (0x020 => driver_features: WriteOnly<u32, DeviceFeatures::Register>),
        (0x024 => driver_features_sel: WriteOnly<u32>),
        (0x028 => @END),
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct VirtIONode {
    interrupts: u8,
    reg: PhysAddrRange,
}

const VIRTIO_NODES: [VirtIONode; 1] = [
    // virtio_mmio@a000000 {
    // 		dma-coherent;
    // 		interrupts = <0x00 0x10 0x01>;
    // 		reg = <0x00 0xa000000 0x00 0x200>;
    // 		compatible = "virtio,mmio";
    // 	};
    VirtIONode {
        interrupts: 0x10,
        reg: PhysAddrRange::fixed(PhysAddr::fixed(0xa000000), 0x200),
    },
];

/// Initialise virtio devices.
pub fn init() -> Result<()> {
    use alloc::collections::BTreeMap;

    let mut device_pages: BTreeMap<PhysAddr, FixedOffset> = BTreeMap::new();

    for node in VIRTIO_NODES.iter() {
        debug!("initialising {:?}", node);
        let page_base = node.reg.base().align_down(PAGESIZE_BYTES);

        if !device_pages.contains_key(&page_base) {
            let virt_addr = Pager::map_device(PhysAddrRange::page_at(page_base))?;
            let translation = FixedOffset::new(page_base, virt_addr.base());
            device_pages.insert(page_base, translation);
        };

        let translation = device_pages.get(&page_base).unwrap();
        let virt_addr = translation.translate_phys(node.reg.base())?;
        let device: &VirtIODevice = unsafe { virt_addr.as_ref() };

        if device.magic_value.read(MagicValue::MAGIC) == MagicValue::MAGIC::Magic.value
            && device.version.get() == 2
        {
            debug!("version 2 device found at {:?}", node.reg);
            if device.device_id.read(DeviceID::TYPE) == DeviceID::TYPE::Block.value {
                let block_device = block::init(node, device)?;
                super::BLOCK_DEVICES
                    .lock()
                    .insert(block_device.name(), block_device);
            }
        }
    }

    Ok(())
}
