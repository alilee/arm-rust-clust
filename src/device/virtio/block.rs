// SPDX-License-Identifier: Unlicense

//! A module for virtio block devices.

use super::queue;
use super::{DeviceID, FeaturesSelect, MagicValue, Status, VirtIODevice};

use crate::device::{RequestId, RequestStatus, Sector};
use crate::pager::{Addr, OwnedMapping, Pager, Paging, PhysAddr, VirtAddr, PAGESIZE_BYTES};
use crate::{Error, Result};

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use core::mem;
use core::pin::Pin;
use core::sync::atomic;
use core::sync::atomic::Ordering;

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::{register_bitfields, register_structs, LocalRegisterCopy};

register_bitfields! [u32,
    BlockDeviceFeaturesLo32 [
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

        /// Ring features
        RING_INDIRECT_DESC  28,
        RING_EVENT_IDX      29,
    ],
    BlockDeviceFeaturesHi32 [
        /// Protocol
        VERSION_1           0,
        ACCESS_PLATFORM     1,

        /// Ring features
        RING_PACKED         2,
        IN_ORDER            3,
        ORDER_PLATFORM      4,
        SR_IOV              5,
        NOTIFICATION_DATA   6,
    ]
];

#[derive(Debug)]
#[repr(C)]
struct Geometry {
    cylinders: u16,
    heads: u8,
    sectors: u8,
}

#[derive(Debug)]
#[repr(C)]
struct Topology {
    physical_block_exp: u8,
    alignment_offset: u8,
    min_io_size: u16,
    opt_io_size: u32,
}

#[derive(Debug)]
#[repr(C)]
struct ConfigLayout {
    capacity: u64,
    size_max: u32,
    seg_max: u32,
    geometry: Geometry,
    blk_size: u32,
    topology: Topology,
    writeback: u8,
    _unused: [u8; 3],
    max_discard_sectors: u32,
    max_discard_seg: u32,
    discard_sector_alignment: u32,
    max_write_zeroes_sectors: u32,
    max_write_zeroes_seg: u32,
    write_zeroes_may_unmap: u8,
    _unused1: [u8; 3],
}

register_structs! {
    VirtIOBlockDevice {
        (0x000 => virtio_device: VirtIODevice),
        (0x100 => config: ConfigLayout),
        (0x140 => @END),
    }
}

type Features = (
    LocalRegisterCopy<u32, BlockDeviceFeaturesHi32::Register>,
    LocalRegisterCopy<u32, BlockDeviceFeaturesLo32::Register>,
);

fn get_device_features(device: &VirtIODevice) -> Result<Features> {
    device
        .device_features_sel
        .write(FeaturesSelect::SELECTION::Lo32);
    let lo32: LocalRegisterCopy<u32, BlockDeviceFeaturesLo32::Register> =
        LocalRegisterCopy::new(device.device_features.get());

    device
        .device_features_sel
        .write(FeaturesSelect::SELECTION::Hi32);
    let hi32 = LocalRegisterCopy::new(device.device_features.get());

    Ok((hi32, lo32))
}

fn negotiate_features(block_device: &VirtIOBlockDevice) -> Result<Features> {
    use BlockDeviceFeaturesHi32::*;
    use BlockDeviceFeaturesLo32::*;

    debug!("config: {:?}", block_device.config);

    let device = &block_device.virtio_device;

    let mut driver_features_lo32 =
        LocalRegisterCopy::<u32, BlockDeviceFeaturesLo32::Register>::new(0);
    driver_features_lo32.write(
        BLK_SIZE_MAX::SET
            + BLK_SEG_MAX::SET
            + BLK_GEOMETRY::SET
            + BLK_BLK_SIZE::SET
            + BLK_FLUSH::SET
            + BLK_TOPOLOGY::SET
            + BLK_CONFIG_WCE::SET
            + BLK_DISCARD::SET
            + BLK_WRITE_ZEROES::SET,
    );
    let mut driver_features_hi32 =
        LocalRegisterCopy::<u32, BlockDeviceFeaturesHi32::Register>::new(0);
    driver_features_hi32.write(VERSION_1::SET + RING_PACKED::SET);
    info!(
        "driver: 0b{:032b} {:032b}",
        driver_features_hi32.get(),
        driver_features_lo32.get()
    );

    let (device_features_hi32, device_features_lo32) = get_device_features(device)?;
    info!(
        "device: 0b{:032b} {:032b}",
        device_features_hi32.get(),
        device_features_lo32.get()
    );

    let overlapping_features_lo32 = device_features_lo32.get() & driver_features_lo32.get();
    let overlapping_features_hi32 = device_features_hi32.get() & driver_features_hi32.get();
    info!(
        "overlp: 0b{:032b} {:032b}",
        overlapping_features_hi32, overlapping_features_lo32
    );

    device
        .driver_features_sel
        .write(FeaturesSelect::SELECTION::Lo32);
    device.driver_features.set(overlapping_features_lo32);
    atomic::fence(atomic::Ordering::SeqCst);

    device
        .driver_features_sel
        .write(FeaturesSelect::SELECTION::Hi32);
    device.driver_features.set(overlapping_features_hi32);
    atomic::fence(atomic::Ordering::SeqCst);

    Ok((
        LocalRegisterCopy::new(overlapping_features_hi32),
        LocalRegisterCopy::new(overlapping_features_lo32),
    ))
}

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
enum RequestType {
    /// Read
    In = 0,
    /// Write
    Out = 1,
    /// Flush
    Flush = 4,
    /// Discard
    Discard = 11,
    /// Write zeroes
    WriteZeroes = 13,
    /// Init
    Init = !0,
}

#[derive(Copy, Clone, Debug)]
#[repr(C, align(32))]
struct ReqHeader {
    req_type: RequestType,
    _reserved: u32,
    sector: Sector,
}

impl Default for ReqHeader {
    fn default() -> Self {
        Self {
            req_type: RequestType::Init,
            _reserved: 0,
            sector: Sector(!0),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, align(32))]
struct Request {
    header: ReqHeader,
    status: RequestStatus,
    init_len: u32,
    id: RequestId,
    descriptor_count: u16,
    used: bool,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            header: ReqHeader::default(),
            status: RequestStatus::Init,
            init_len: !0,
            id: RequestId(!0),
            descriptor_count: 3,
            used: false,
        }
    }
}

impl Request {
    pub fn header_descriptor(&self) -> Result<(PhysAddr, u32)> {
        Ok((
            Pager::maps_to((&self.header).into())?,
            mem::size_of_val(&self.header) as u32,
        ))
    }

    pub fn status_descriptor(&self) -> Result<(PhysAddr, u32)> {
        Ok((
            Pager::maps_to((&self.status).into())?,
            mem::size_of_val(&self.status) as u32,
        ))
    }
}

struct BlockDevice<'a> {
    name: String,
    features: Features,
    mmio_range: Arc<OwnedMapping>,
    regs: &'a VirtIOBlockDevice,
    interrupt: u32,
    virt_queue: queue::PackedRing<'a>,
    requests: BTreeMap<RequestId, Pin<Box<Request>>>,
}

impl<'a> BlockDevice<'a> {
    fn new(
        name: &str,
        features: Features,
        mmio_range: Arc<OwnedMapping>,
        regs: &'a VirtIOBlockDevice,
        interrupt: u32,
        virt_queue: queue::PackedRing<'a>,
    ) -> Self {
        Self {
            name: name.to_string(),
            features,
            mmio_range,
            regs,
            interrupt,
            virt_queue,
            requests: BTreeMap::new(),
        }
    }
}

/// tock_registers structs are implemented using UnsafeCells are not Send.
///
/// The reference to the unsafe cells are safe to Send because underlying
/// u32 is safe to Send. Ok so I don't really know this stuff.
unsafe impl Send for BlockDevice<'_> {}

impl crate::device::Block for BlockDevice<'_> {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn status(&mut self, id: RequestId) -> Result<u32> {
        let req = self.requests.get(&id).ok_or(Error::UnexpectedValue)?;
        dbg!(&req);
        while let Ok(desc) = self.virt_queue.next_used() {
            dbg!(desc);
            let id = desc.id;
            let req = self.requests.get_mut(&id).ok_or(Error::UnexpectedValue)?;
            req.used = true;
            req.init_len = desc.len;
            self.virt_queue
                .consume_used(req.descriptor_count as usize)?;
        }
        let req = self.requests.get(&id).ok_or(Error::UnexpectedValue)?;
        dbg!(&req);
        if req.used {
            match req.status {
                RequestStatus::Ok => Ok(req.init_len - 1), // less one byte for status
                RequestStatus::IOErr => Err(Error::IOError),
                RequestStatus::Unsupp => Err(Error::DeviceIncompatible),
                RequestStatus::Init => unreachable!(),
            }
        } else {
            Err(Error::WouldBlock)
        }
    }

    fn read(&mut self, page_addrs: &[PhysAddr], sector: Sector) -> Result<RequestId> {
        let id = self.virt_queue.next(3)?;
        let request = Box::pin(Request {
            header: ReqHeader {
                req_type: RequestType::In,
                sector,
                ..Default::default()
            },
            ..Default::default()
        });

        dbg!(&request);

        let read_ranges = vec![request.header_descriptor()?];
        let mut write_ranges: Vec<(PhysAddr, u32)> = page_addrs
            .iter()
            .map(|phys_addr| (phys_addr.clone(), PAGESIZE_BYTES as u32))
            .collect();
        write_ranges.push(request.status_descriptor()?);

        self.virt_queue.submit(id, &read_ranges, &write_ranges)?;
        atomic::fence(Ordering::SeqCst);
        self.regs
            .virtio_device
            .queue_notify
            .set(self.virt_queue.index());

        self.requests.insert(id, request);
        Ok(id)
    }

    fn write(&mut self, page_addrs: &[PhysAddr], sector: Sector) -> Result<RequestId> {
        todo!()
    }

    fn discard(&mut self, sector: Sector, length: usize) -> Result<RequestId> {
        todo!()
    }

    fn zero(&mut self, sector: Sector, length: usize) -> Result<RequestId> {
        todo!()
    }

    fn flush(&mut self) -> Result<RequestId> {
        todo!()
    }
}

/// Attempt to initialise a block device behind a virtio node.
///
/// NOTE:
#[inline(never)]
pub(in crate::device::virtio) fn init(
    name: &str,
    mmio_range: Arc<OwnedMapping>,
    reg: VirtAddr,
    interrupt: u32,
) -> Result<Box<dyn crate::device::Block + Send>> {
    major!("init {:?}: {:?}, {:?}", name, reg, interrupt);
    let block_device: &mut VirtIOBlockDevice = unsafe { reg.as_mut_ref() };

    let negotiated_features = {
        let device = &mut block_device.virtio_device;

        device.status.set(0);
        atomic::fence(atomic::Ordering::SeqCst);

        assert_eq!(MagicValue::MAGIC::Magic.value, device.magic_value.get(),);
        assert_eq!(2, device.version.get(),);

        device.status.modify(Status::ACKNOWLEDGE::SET);
        atomic::fence(atomic::Ordering::SeqCst);
        info!("status: 0b{:32b}", device.status.get());

        assert_eq!(
            DeviceID::TYPE::Block.value,
            device.device_id.read(DeviceID::TYPE)
        );
        assert_eq!(reg.increment(0x70), VirtAddr::from(&device.status));

        device.status.modify(Status::DRIVER::SET);
        atomic::fence(atomic::Ordering::SeqCst);
        info!("status: 0b{:32b}", device.status.get());

        negotiate_features(&block_device)?
    };

    let virt_queue = {
        let device = &mut block_device.virtio_device;

        device.status.modify(Status::FEATURES_OK::SET);
        atomic::fence(atomic::Ordering::SeqCst);
        info!("status: 0b{:32b}", device.status.get());

        if !device.status.is_set(Status::FEATURES_OK) {
            return Err(Error::DeviceIncompatible);
        }

        let mut virt_queue = queue::PackedRing::new(0)?;
        device.add_queue(&mut virt_queue)?;
        dbg!(&virt_queue);

        device.status.modify(Status::DRIVER_OK::SET);
        atomic::fence(atomic::Ordering::SeqCst);
        info!("status: 0b{:32b}", device.status.get());

        virt_queue
    };

    Ok(Box::new(BlockDevice::new(
        name,
        negotiated_features,
        mmio_range,
        block_device,
        interrupt,
        virt_queue,
    )))
}
