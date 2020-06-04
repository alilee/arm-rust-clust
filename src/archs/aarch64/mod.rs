// SPDX-License-Identifier: Unlicense

mod device;
mod handler;
mod pager;

/// Live hardware abstraction layer for integration tests and releases.
#[cfg(not(test))]
mod hal;

/// Mock hardware abstraction layer for unit tests.
#[cfg(test)]
mod hal_test;

/// Publish hardware abstraction layer for unit tests.
#[cfg(test)]
use hal_test as hal;

use crate::pager::{Addr, AddrRange, PhysAddr, PhysAddrRange, VirtAddr};
use crate::Result;

/// Materialise empty struct implementing Arch trait.
pub struct Arch {}

#[cfg(not(test))]
pub use hal::_reset;

/// Construct an empty page directory.
pub fn new_page_directory() -> impl super::PageDirectory {
    pager::new_page_directory()
}

#[cfg(test)]
mod tests {
    extern crate std;

    #[test]
    fn it_works() {
        info!("marker")
    }
}
