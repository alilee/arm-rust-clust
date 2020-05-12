// SPDX-License-Identifier: Unlicense

//! Unifies the CPU-architecture specific code for the supported CPU's.
//!
//! All architectures are linked in for unit testing on the host platform.
//! Only the target architecture is linked for release builds (including when
//! used for integration testing).
//!
//! The target architecture for the build is usable at archs::arch

use crate::Result;
use crate::util::locked::Locked;
use crate::{pager, pager::{PhysAddrRange, VirtAddr}};

/// Each architecture must supply the following entry points.
pub trait ArchTrait {
    /// Physical address range of ram
    fn ram_range() -> Result<PhysAddrRange>;
    /// Base virtual address of kernel address space
    fn kernel_base() -> VirtAddr;

    /// Initialise virtual memory management.
    fn pager_init() -> Result<()>;
    /// Map physical address range at offset
    fn map_translation(
        phys_range: pager::PhysAddrRange,
        virtual_address_translation: impl pager::Translate,
        attrs: pager::Attributes,
        allocator: &Locked<impl pager::FrameAllocator>,
        mem_access_translation: impl pager::Translate,
    );
    /// Map physical address range at offset
    fn map_demand(
        virtual_range: pager::VirtAddrRange,
        attrs: pager::Attributes,
        allocator: &Locked<impl pager::FrameAllocator>,
        mem_access_translation: impl pager::Translate,
    );
    /// Enable virtual memory management.
    fn enable_paging();

    /// Initialise the exception handler.
    fn handler_init() -> Result<()>;

    /// Initialise tasking and multi-processing.
    fn thread_init() -> Result<()>;

    /// Initialise tasking and multi-processing.
    fn wait_forever() -> !;
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
