// SPDX-License-Identifier: Unlicense

//! Kernel as library, to facilitate integration testing.

#![no_std]

#![feature(naked_functions)] // for _reset
#![feature(panic_info_message)]
#![feature(format_args_nl)]  // for debug logging macros
#![feature(const_fn)] // casting pointer to ints in PhysAddr::from_linker_symbol
#![feature(const_raw_ptr_to_usize_cast)] // casting pointer to ints in PhysAddr::from_linker_symbol
#![feature(linkage)] // for weak linkage of panic::_panic_exit
#![feature(const_transmute)] // for virt addr mem::transmute
#![feature(const_if_match)] // for assertions in const functions (eg. VirtAddr::align_up)
#![feature(const_panic)] // for assertions in const functions (eg. VirtAddr::align_up)
#![feature(asm)] // used throughout archs
#![feature(global_asm)] // for exception handler and return

#![warn(missing_docs)]

#[macro_use]
pub mod debug;

pub mod archs;
pub mod device;
pub mod handler;
pub mod pager;
pub mod util;

mod panic;

pub use util::result::{Result, Error};

#[allow(unused_imports)]
use crate::archs::arch::_reset;

// extern crate alloc;

#[macro_use]
extern crate claim;
