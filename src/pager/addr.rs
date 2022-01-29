// SPDX-License-Identifier: Unlicense

//! Managing virtual address space, address translation and page faults.

use crate::pager::PAGESIZE_BYTES;

/// A usize value representing either a virtual or physical address.
pub trait Addr<A: Addr<A, R> + core::marker::Copy + core::clone::Clone, R: AddrRange<A, R>> {
    /// Lowest possible address.
    fn null() -> A
    where
        Self: Sized,
    {
        A::at(0)
    }

    /// Construct at literal address.
    fn at(addr: usize) -> A
    where
        Self: Sized;

    /// Get the offset from memory base in bytes.
    fn get(&self) -> usize;

    /// Get the offset from memory base as a pair of u32.
    fn hilo(&self) -> (u32, u32) {
        let p = self.get();
        ((p >> 32) as u32, (p & (u32::MAX as usize)) as u32)
    }

    /// Number of bytes above reference point.
    fn offset_above(&self, base: A) -> usize {
        self.get() - base.get()
    }

    /// Number of bytes below reference point.
    fn offset_below(&self, top: A) -> usize {
        top.get() - self.get()
    }

    /// An address that is lower than this by a given number of bytes.
    fn decrement(&self, offset: usize) -> A {
        A::at(self.get() - offset)
    }

    /// An address that is higher than this by a given number of bytes.
    fn increment(&self, offset: usize) -> A {
        use core::intrinsics::unchecked_add;

        let top = unsafe { unchecked_add(self.get(), offset) };
        if top >= self.get() {
            A::at(top)
        } else {
            A::null()
        }
    }

    /// Aligned on a byte boundary.
    fn is_aligned(&self, boundary: usize) -> bool {
        assert!(boundary.is_power_of_two());
        0 == self.get() & (boundary - 1)
    }

    /// Nearest higher address with given alignment.
    fn align_up(&self, boundary: usize) -> A {
        assert!(boundary.is_power_of_two());
        A::at(self.get() + (boundary - 1) & !(boundary - 1))
    }

    /// Nearest lower address with given alignment.
    fn align_down(&self, boundary: usize) -> A {
        assert!(boundary.is_power_of_two());
        A::at(self.get() & !(boundary - 1))
    }

    /// Address of page containing address.
    fn page_base(&self) -> A {
        self.align_down(PAGESIZE_BYTES)
    }

    /// Addresses offset within page.
    fn page_offset(&self) -> usize {
        self.get() & (PAGESIZE_BYTES - 1)
    }
}

/// A contiguous range bounded by two of either virtual or physical addresses.
pub trait AddrRange<A: Addr<A, R> + core::marker::Copy + core::clone::Clone, R: AddrRange<A, R>> {
    /// Construct from base and length.
    fn new(base: A, length: usize) -> R;

    /// Construct from base with one page width
    fn page_at(base: A) -> R {
        R::new(base, PAGESIZE_BYTES)
    }

    /// Single page containing given address
    fn page_containing(addr: A) -> R {
        R::page_at(addr.page_base())
    }

    /// Range between two addresses.
    fn between(base: A, top: A) -> R {
        R::new(base, top.offset_above(base))
    }

    /// Get the base of the range.
    fn base(&self) -> A;

    /// Length of the range in bytes.
    fn length(&self) -> usize;

    /// Address at the top of the range.
    fn top(&self) -> A {
        A::at(self.base().get() + self.length())
    }

    /// Is range aligned base and top aligned at boundary.
    fn is_aligned(&self, boundary: usize) -> bool {
        0 == (self.base().get() | self.length()) & (boundary - 1)
    }

    /// Range same base different length.
    fn resize(&self, length: usize) -> R {
        R::new(self.base(), length)
    }

    /// Length of the range in bytes.
    ///
    /// Written relative to length to avoid adding top and wrapping at top of memory.
    fn intersection(&self, other: &Self) -> Option<R> {
        use core::cmp::min;

        let (lower, higher) = if self.base().get() <= other.base().get() {
            (self, other)
        } else {
            (other, self)
        };

        let distance_above = higher.base().get() - lower.base().get();
        if distance_above > lower.length() {
            return None;
        }
        let overlap_length = lower.length() - distance_above;
        Some(R::new(higher.base(), min(higher.length(), overlap_length)))
    }

    /// Increment the range by its length.
    fn step(self: &Self) -> R {
        Self::new(self.base().increment(self.length()), self.length())
    }

    /// True iff the range includes the address.
    fn contains(&self, addr: A) -> bool {
        addr.get() >= self.base().get() && addr.get() < self.top().get()
    }

    /// True iff the range extends to at least the base and top of other.
    fn covers(&self, other: &Self) -> bool {
        self.base().get() <= other.base().get() && self.top().get() >= other.top().get()
    }
}
