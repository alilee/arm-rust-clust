// SPDX-License-Identifier: Unlicense

pub mod mair {
    use crate::Result;

    pub fn init() -> Result<()> {
        trace!("init");
        Ok(())
    }
}