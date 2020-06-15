// SPDX-License-Identifier: Unlicense

//! Exception handling trait for aarch64.

use super::{hal, Arch};

use crate::archs::HandlerTrait;
use crate::Result;

impl HandlerTrait for Arch {
    fn handler_init() -> Result<()> {
        info!("init");
        hal::set_vbar()
    }
}
