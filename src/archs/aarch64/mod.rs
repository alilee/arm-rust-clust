// SPDX-License-Identifier: Unlicense

#[cfg(test)]
pub mod hal_test;

#[cfg(test)]
pub use hal_test as hal;

#[cfg(not(test))]
pub mod hal_live;

#[cfg(not(test))]
pub use hal_live as hal;

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