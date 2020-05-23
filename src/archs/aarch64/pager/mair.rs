// SPDX-License-Identifier: Unlicense

//! Memory Attribute Indirection Register
//!
//! Sets up the list of memory access types and publishes the valid
//! list as an enum.

use crate::Result;

#[derive(Debug)]
pub enum MAIR {
    DeviceStronglyOrdered = 0,
    MemoryWriteThrough,
}

impl From<u64> for MAIR {
    fn from(i: u64) -> Self {
        use MAIR::*;
        match i {
            0 => DeviceStronglyOrdered,
            1 => MemoryWriteThrough,
            _ => panic!("unknown MAIR conversion"),
        }
    }
}

pub fn init() -> Result<()> {
    crate::archs::aarch64::hal::mair::init()
}
