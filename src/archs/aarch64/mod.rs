// SPDX-License-Identifier: Unlicense

/// Mock hardware abstraction layer for unit tests.
#[cfg(test)]
pub mod hal_test;

/// Publish hardware abstration layer for unit tests.
#[cfg(test)]
pub use hal_test as hal;

/// Live hardware abstraction layer for integration tests and releases.
#[cfg(not(test))]
pub mod hal_live;

/// Publish hardware abstration layer for integration tests and releases.
#[cfg(not(test))]
pub use hal_live as hal;

/// Materialise empty struct implementating Arch trait.
pub struct Arch {}

impl super::ArchTrait for Arch {
    fn init_handler() {
        1;
    }
    fn init_pager() {
        2;
    }
    fn init_thread() {
        3;
    }
}

#[cfg(not(test))]
pub use hal::_reset;

#[cfg(test)]
mod tests {
    extern crate std;
    use std::dbg;

    #[test]
    fn sandwich() {
        dbg!("hello");
        assert!(true)
    }
}