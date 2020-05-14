// SPDX-License-Identifier: Unlicense

//! Types for managing pages.

use crate::pager::PAGESIZE_BYTES;

/// A 4k page.
#[derive(Copy, Clone)]
#[repr(align(4096))]
pub struct Page([u8; PAGESIZE_BYTES]);

impl Page {
    /// Any empty page.
    pub const fn new() -> Self {
        Self([0u8; PAGESIZE_BYTES])
    }
}