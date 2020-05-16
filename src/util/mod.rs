// SPDX-License-Identifier: Unlicense

//! Miscellaneous support functions.

pub mod locked;
pub mod result;

#[cfg(not(test))]
pub mod testing;