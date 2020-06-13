// SPDX-License-Identifier: Unlicense

use crate::pager::Translate;
use crate::Result;

pub mod mair {
    use crate::Result;

    pub fn init() -> Result<()> {
        trace!("init");
        Ok(())
    }
}

pub fn enable_paging(_: u64, _: u64, _: u16, _: usize) -> Result<()> {
    unimplemented!()
}

pub fn set_vbar(_: impl Translate) -> Result<()> {
    unimplemented!()
}
