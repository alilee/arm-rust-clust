// SPDX-License-Identifier: Unlicense

//! Miscellaneous support functions.

pub mod bitfield;
pub mod locked;
pub mod result;

#[cfg(not(test))]
pub mod testing;