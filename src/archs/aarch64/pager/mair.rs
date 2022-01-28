// SPDX-License-Identifier: Unlicense

//! Memory Attribute Indirection Register
//!
//! Sets up the list of memory access types and publishes the valid
//! list as an enum.

use super::hal;

use crate::Result;

/// Type for indexes into the MAIR register (referenced in page table).
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

/// Initialise the MAIR register.
pub fn init() -> Result<()> {
    info!("init");
    hal::init_mair();
    Ok(())
}
