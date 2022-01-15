// SPDX-License-Identifier: Unlicense

//! Kernel as library, to facilitate integration testing.

#![no_std]
#![feature(naked_functions)] // for _reset
#![feature(panic_info_message)]
#![feature(format_args_nl)] // for debug logging macros
#![feature(linkage)] // for weak linkage of panic::_panic_exit
#![feature(core_intrinsics)] // for unchecked_sub in checking perms for ptes
#![feature(alloc_error_handler)] // for kernel heap
#![feature(const_mut_refs)] // for as_mut_ref
#![feature(const_fn_trait_bound)] // for BitField::new
#![feature(const_btree_new)] // for device-name maps
#![feature(variant_count)] // for page frame deque
#![warn(missing_docs)]

#[macro_use]
pub mod debug;

pub mod archs;
pub mod device;
pub mod handler;
pub mod pager;
pub mod thread;
pub mod util;

mod panic;

pub use util::result::{Error, Result};

#[allow(unused_imports)]
use crate::archs::arch::reset;

extern crate alloc;

#[macro_use]
extern crate claim;
