// SPDX-License-Identifier: Unlicense

//! Types for managing pages.

use crate::pager::PAGESIZE_BYTES;
use core::ops::{Index, IndexMut};

/// A 4k page.
#[derive(Copy, Clone)]
#[repr(align(4096))]
pub struct Page([u64; PAGESIZE_BYTES / 8]);

impl Page {
    /// Any empty page.
    pub const fn new() -> Self {
        Self([0u64; PAGESIZE_BYTES / 8])
    }
}

impl Index<usize> for Page {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        assert_lt!(index, PAGESIZE_BYTES / 8);
        &self.0[index]
    }
}

impl IndexMut<usize> for Page {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert_lt!(index, PAGESIZE_BYTES / 8);
        &mut self.0[index]
    }
}
