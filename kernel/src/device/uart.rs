//! A stream sink which writes to the serial port.

use crate::pager::{PhysAddr, PhysAddrRange, PAGESIZE_BYTES};
use crate::util::locked::Locked;
use core::fmt;
use core::fmt::{write, Arguments, Write};

/// Represents a UART end-point.
pub struct Uart {
    phys_addr: PhysAddr,
    dr_addr: *mut u32,
}

impl Uart {
    /// Create a Uart structure for UART0 id_mapped.
    pub const fn uart0() -> Uart {
        let phys_addr = PhysAddr::new_const(0x09000000);
        let dr_addr = PhysAddr::new_const(0x9000000).identity_map_mut() as *mut u32;
        Uart { phys_addr, dr_addr }
    }

    pub fn reset(&mut self) -> Result<(), u64> {
        use crate::pager;
        self.dr_addr = pager::device_map::<u32>(self.phys_addr)?;
        Ok(())
    }

    /// Write one byte to the Uart.
    fn put(&self, b: u8) {
        unsafe {
            *self.dr_addr = b as u32;
        }
    }

    pub fn phys_addr(&self) -> PhysAddrRange {
        PhysAddrRange::new(self.phys_addr, PAGESIZE_BYTES)
    }
}

unsafe impl Sync for Uart {}
unsafe impl Send for Uart {}

pub static UART0: Locked<Uart> = Locked::new(Uart::uart0());

impl Write for Uart {
    /// Writes a slice of bytes to Uart, as stream for formatted output.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.as_bytes() {
            self.put(*b)
        }
        Ok(())
    }
}

pub fn _dbg_writer(args: Arguments) {
    write(&mut (*UART0.lock()), args).unwrap();
}
