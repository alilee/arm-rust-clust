// SPDX-License-Identifier: Unlicense

mod handler;
mod pager;
mod reset;

pub use handler::*;
pub use pager::*;
pub use reset::*;

#[inline(always)]
/// Unique identifier for each core
pub fn core_id() -> u8 {
    use cortex_a::registers::{MPIDR_EL1, MPIDR_EL1::AFF0};
    use tock_registers::interfaces::Readable;

    MPIDR_EL1.read(AFF0) as u8
}
