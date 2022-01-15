// SPDX-License-Identifier: Unlicense

mod device;
mod handler;
mod pager;

const UPPER_VA_BITS: usize = 39; // 512 GB, avoids 1 level
const LOWER_VA_BITS: usize = 48; // 256 TB

/// Live hardware abstraction layer for integration tests and releases.
#[cfg(not(test))]
mod hal;

/// Mock hardware abstraction layer for unit tests.
#[cfg(test)]
mod hal_test;

/// Publish hardware abstraction layer for unit tests.
#[cfg(test)]
use hal_test as hal;

/// Materialise empty struct implementing Arch trait.
pub struct Arch {}

#[cfg(not(test))]
pub use hal::reset;

/// Construct an empty page directory.
pub fn new_page_directory() -> impl super::PageDirectory {
    pager::new_page_directory()
}

#[cfg(not(test))]
pub use hal::core_id;

#[cfg(test)]
pub use hal_test::core_id;

pub use pager::PageBlockDescriptor;
pub use pager::PageDirectory;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use() {
        assert_eq!(LOWER_VA_BITS, 48)
    }
}
