// SPDX-License-Identifier: Unlicense

//! A stream sink which writes to the serial port.

use crate::Result;
use crate::pager::{PhysAddr, Translate};

use core::fmt;
use core::fmt::Write;

/// Represents a UART end-point.
pub struct Uart {
    dr_addr: *mut u32, // data register
}

unsafe impl Sync for Uart {}
unsafe impl Send for Uart {}

impl Uart {
    /// Create a Uart structure for UART0 id_mapped.
    pub const fn debug() -> Uart {
        // FIXME: Find UART in device tree
        let dr_addr = 0x9000000 as *mut u32;
        Uart { dr_addr }
    }

    /// Write one byte to the Uart.
    fn put(&self, b: u8) {
        unsafe {
            *self.dr_addr = b as u32;
        }
    }

    /// Remap Uart structure.
    ///
    /// UNSAFE: Register pointer must point to physical memory.
    #[allow(unused_unsafe)]
    pub unsafe fn translate(&mut self, remap: impl Translate) -> Result<()> {
        let pa = unsafe { PhysAddr::from_ptr(self.dr_addr as *const u8) };
        let va = remap.translate(pa);
        self.dr_addr = va.into();
        Ok(())
    }
}

impl Write for Uart {
    /// Writes a slice of bytes to Uart, as stream for formatted output.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.as_bytes() {
            self.put(*b)
        }
        Ok(())
    }
}
