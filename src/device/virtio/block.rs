// SPDX-License-Identifier: Unlicense

//! A module for virtio block devices.

use super::{DeviceID, VirtIODevice};

use crate::pager::{Addr, VirtAddr};
use crate::{Error, Result};

use alloc::boxed::Box;
use tock_registers::interfaces::Readable;

/// Attempt to initialise a block device behind a virtio node.
///
/// NOTE:
pub(in crate::device::virtio) fn init(
    reg: VirtAddr,
    interrupt: u32,
    device: &VirtIODevice,
) -> Result<Box<dyn crate::device::Block>> {
    info!("init {:?}, {:?}", reg, interrupt);
    assert_eq!(
        DeviceID::TYPE::Block.value,
        device.device_id.read(DeviceID::TYPE)
    );

    Err(Error::DeviceIncompatible)
}
