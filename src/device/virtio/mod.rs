// SPDX-License-Identifier: Unlicense

//! A module for virtio devices.

mod block;

use crate::pager::{
    Addr, AddrRange, FixedOffset, Pager, Paging, PhysAddr, PhysAddrRange, Translate, PAGESIZE_BYTES,
};
use crate::{Error, Result};

use tock_registers::interfaces::Readable;
use tock_registers::registers::{ReadOnly, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

use dtb::StructItem;

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

fn make_addr(prop: StructItem) -> Result<PhysAddr> {
    let mut buf = [0u8; 32];
    let list = prop
        .value_u32_list(&mut buf)
        .or(Err(Error::DeviceIncompatible))?;
    Ok(PhysAddr::fixed(
        (list[0] as usize) << 32 | (list[1] as usize),
    ))
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

    let mut device_pages: BTreeMap<PhysAddr, FixedOffset> = BTreeMap::new();

    for (node, node_iter) in dtb_root.path_struct_items("/virtio_mmio") {
        let (prop, _) = node_iter.clone().path_struct_items("reg").next().unwrap();
        let phys_addr = make_addr(prop)?;
        let page_base = phys_addr.align_down(PAGESIZE_BYTES);

        if !device_pages.contains_key(&page_base) {
            let virt_addr = Pager::map_device(PhysAddrRange::page_at(page_base))?;
            let translation = FixedOffset::new(page_base, virt_addr.base());
            device_pages.insert(page_base, translation);
        };

        let translation = device_pages.get(&page_base).unwrap();
        let reg = translation.translate_phys(phys_addr)?;
        let device: &VirtIODevice = unsafe { reg.as_ref() };

        if device.magic_value.read(MagicValue::MAGIC) == MagicValue::MAGIC::Magic.value
            && device.version.get() == 2
        {
            debug!(
                "version 2 device ({:?}) found at {:?}",
                device.device_id.read(DeviceID::TYPE),
                node
            );
            match device.device_id.read_as_enum(DeviceID::TYPE) {
                Some(DeviceID::TYPE::Value::Invalid) => {}
                Some(DeviceID::TYPE::Value::Block) => {
                    let (interrupt, _) = node_iter
                        .clone()
                        .path_struct_items("interrupts")
                        .next()
                        .unwrap();
                    let interrupt = make_intr(interrupt)?;
                    let block_device = block::init(reg, interrupt.1, device)?;
                    super::BLOCK_DEVICES
                        .lock()
                        .insert(block_device.name(), block_device);
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
                    unimplemented!()
                }
            };
        }
    }

    Ok(())
}
