// SPDX-License-Identifier: Unlicense

//! Unifies the CPU-architecture specific code for the supported CPU's.
//!
//! All architectures are linked in for unit testing on the host platform.
//! Only the target architecture is linked for release builds (including when
//! used for integration testing).
//!
//! The target architecture for the build is usable at archs::arch

/// Each architecture must supply the following entry points.
pub trait ArchTrait {
    /// Initialise the exception handler.
    fn init_handler();
    /// Initialise virtual memory management.
    fn init_pager();
    /// Initialise tasking and multi-processing.
    fn init_thread();
}

/// A mock architecture for use during unit testing.
#[cfg(test)]
pub mod test;

/// ARM architecture v8 (64-bit)
#[cfg(any(test, target_arch = "aarch64"))]
pub mod aarch64;

/// Intel/AMD architecture 64-bit
#[cfg(any(test, target_arch = "x86_64"))]
pub mod x86_64;

// publish the target arch at Arch
#[cfg(test)]
pub use test as arch;

#[cfg(all(not(test), target_arch = "aarch64"))]
pub use aarch64 as arch;

#[cfg(all(not(test), target_arch = "x86_64"))]
pub use x86_64 as arch;
