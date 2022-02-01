// SPDX-License-Identifier: Unlicense

//! Types for managing pages.

use crate::pager::PAGESIZE_BYTES;

/// A 4k page.
#[derive(Copy, Clone)]
#[repr(align(4096))]
pub struct Page([u64; PAGESIZE_BYTES / 8]);

impl Page {
    /// Any empty page.
    pub const fn new() -> Self {
        Self([0u64; PAGESIZE_BYTES / 8])
    }

    ///
    pub fn slice(&mut self) -> &mut [u64] {
        &mut self.0
    }
}
