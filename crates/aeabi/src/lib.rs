#![no_std]

#[no_mangle]
pub unsafe extern fn __aeabi_memclr4(s: *mut u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.offset(i as isize) = 0u8;
        i += 1;
    }
    return s;
}

#[no_mangle]
pub unsafe extern fn __aeabi_uidivmod(dividend: u32, divisor: u32) -> (u32, u32) {

    if divisor == 0 { panic!("divide by zero") }
    
    let mut pos = 0;
    let mut d = divisor;
    while d <= dividend {
        pos += 1;
        d <<= 1;
    } 
    pos -= 1;
    
    let mut quot = 0;
    let mut rem = dividend;
    loop {
        quot |= 1;
        rem -= divisor << pos; 
        loop {
            if pos == 0 { return (quot, rem); }

            quot <<= 1;
            pos -= 1;
            
            if rem >= (divisor << pos) { break; }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    
    #[test]
    fn test_div() {
        unsafe {
            assert_eq!(__aeabi_uidivmod(0b11011, 0b101), (0b101, 0b10));
            assert_eq!(__aeabi_uidivmod(27, 5), (5, 2));
            assert_eq!(__aeabi_uidivmod(25, 5), (5, 0));
            assert_eq!(__aeabi_uidivmod(5, 5), (1, 0));
            assert_eq!(__aeabi_uidivmod(1, 5), (0, 1));
            assert_eq!(__aeabi_uidivmod(0, 5), (0, 0));
            assert_eq!(__aeabi_uidivmod(13, 2), (6, 1));
        }
    }

}