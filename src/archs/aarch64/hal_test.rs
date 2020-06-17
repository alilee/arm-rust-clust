// SPDX-License-Identifier: Unlicense

use crate::Result;

pub mod mair {
    pub fn init() {}
}

pub fn enable_paging(_: u64, _: u64, _: u16) -> Result<()> {
    unimplemented!()
}

pub fn set_vbar() -> Result<()> {
    unimplemented!()
}
