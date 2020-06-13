// SPDX-License-Identifier: Unlicense

//! Exception handling trait for aarch64.

use super::{Arch, hal};

use crate::archs::HandlerTrait;
use crate::pager::Translate;
use crate::Result;

impl HandlerTrait for Arch {
    fn handler_init(translation: impl Translate) -> Result<()> {
        info!("init");
        hal::set_vbar(translation)
    }
}