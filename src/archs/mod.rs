// SPDX-License-Identifier: Unlicense

pub trait ArchTrait {
    fn init_handler();
    fn init_pager();
    fn init_thread();
}

// include just the target arch, or all arches if testing
#[cfg(test)]
pub mod test;

#[cfg(any(test, target_arch = "aarch64"))]
pub mod aarch64;

#[cfg(any(test, target_arch = "x86_64"))]
pub mod x86_64;

// publish the target arch at Arch
#[cfg(test)]
pub use test as arch;

#[cfg(all(not(test), target_arch = "aarch64"))]
pub use aarch64 as arch;

#[cfg(all(not(test), target_arch = "x86_64"))]
pub use x86_64 as arch;
