// SPDX-License-Identifier: Unlicense

use crate::Result;

pub fn init_mair() {}

pub fn enable_paging(_: u64, _: u64, _: u16) -> Result<()> {
    unimplemented!()
}

pub fn move_stack(_: usize) -> () {
    unimplemented!()
}

pub fn set_vbar() -> Result<()> {
    unimplemented!()
}

pub fn core_id() -> u8 {
    1
}