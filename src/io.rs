
/// Address of UART0 transmit register
const UART0DR: *mut u32 = 0x101f1000 as *mut u32;

/// Write one character to the serial
pub fn putc(c: u8) {
    let i = c as u32;
    unsafe {
        *UART0DR = i;
    }
}

/// Write a string of characters to the serial port
pub fn puts(s: &str) {
    for c in s.as_bytes() {
        putc(*c)
    }
}