// SPDX-License-Identifier: Unlicense

//! A module for virtio block devices.

use crate::{Error, Result};
use alloc::boxed::Box;
use tock_registers::interfaces::Readable;

use super::{DeviceID, VirtIODevice, VirtIONode};

/// Attempt to initialise a block device behind a virtio node.
///
/// NOTE:
pub(in crate::device::virtio) fn init(
    _node: &VirtIONode,
    device: &VirtIODevice,
) -> Result<Box<dyn crate::device::Block>> {
    assert_eq!(
        DeviceID::TYPE::Block.value,
        device.device_id.read(DeviceID::TYPE)
    );
    Err(Error::DeviceIncompatible)
}
