// SPDX-License-Identifier: Unlicense

use crate::{Result, Error};
use crate::util::locked::Locked;

use super::{PhysAddr, PhysAddrRange};

/// Ability to provide an unused frame.
pub trait Allocator {
    /// Reserve and return a zero'd frame.
    fn alloc_page(&mut self) -> Result<PhysAddr>;
}

/// Initialise
pub fn init() -> Result<()> {
    info!("init");
    Ok(())
}

/// Extend the frame table to include a range of physical addresses.
pub fn add_ram_range(range: PhysAddrRange) -> Result<()> {
    info!("including: {:?}", range);
    Ok(())
}

/// O(1) allocator of available pages
pub struct StackAllocator {
    _top: Option<PhysAddr>,
}

impl StackAllocator {
    const fn new() -> Self {
        Self {
            _top: None
        }
    }
}

impl Allocator for StackAllocator {
    fn alloc_page(&mut self) -> Result<PhysAddr> {
        Err(Error::UnknownError)
    }
}

pub static ALLOCATOR: Locked<StackAllocator> = Locked::new(StackAllocator::new());
