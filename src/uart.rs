//! A stream sink which writes to the serial port.

use core::fmt::{Write, Result};

/// Represents a UART end-point.
pub struct Uart {
    dr_addr: *mut u32,
}

impl Uart {
    
    /// Create a Uart structure for UART0. 
    pub const fn uart0() -> Uart {
        Uart { dr_addr: 0x09000000 as *mut u32 }
    }
    
    /// Write one byte to the Uart.
    fn put(&self, b: u8) {
        unsafe {
            *self.dr_addr = b as u32;
        }
    }
}

pub const UART0: Uart = Uart::uart0();

impl Write for Uart {
    
    /// Writes a slice of bytes to Uart, as stream for formatted output.
    fn write_str(&mut self, s: &str) -> Result {
        for b in s.as_bytes() {
            self.put(*b)
        }
        Ok(())
    }

}