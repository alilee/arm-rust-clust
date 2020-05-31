// SPDX-License-Identifier: Unlicense

//! Managing virtual address space, address translation and page faults.

/// A usize value representing either a virtual or physical address.
pub trait Addr<A: Addr<A, R> + core::marker::Copy + core::clone::Clone, R: AddrRange<A, R>> {
    /// Lowest possible address.
    fn null() -> A {
        A::at(0)
    }

    /// Construct at literal address.
    fn at(addr: usize) -> A;

    /// Get the offset from memory base in bytes.
    fn get(&self) -> usize;

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
        A::at(self.get() + offset)
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
}

/// A contiguous range bounded by two of either virtual or physical addresses.
pub trait AddrRange<A: Addr<A, R> + core::marker::Copy + core::clone::Clone, R: AddrRange<A, R>> {
    /// Construct from base and length.
    fn new(base: A, length: usize) -> R;

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
}