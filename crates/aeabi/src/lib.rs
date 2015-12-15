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

/// Divide two unsigned words returning quotient and remainder  
/// 
/// http://www.dragonwins.com/domains/getteched/de248/binary_division.htm
#[no_mangle]
pub unsafe extern fn __aeabi_uidivmod(dividend: u32, divisor: u32) -> (u32, u32) {

    let mut quotient = 0u32;
    let mut remainder = dividend;
    let mut term = 1u32;
    let mut product = divisor;
    
    while (term < 0x8000000u32) && (product < remainder) {
        product <<= 1;
        term <<= 1;
    }
    
    loop {
        if product <= remainder {
            remainder -= product;
            quotient += term;
        }
        
        if term == 1u32 { return (quotient, remainder); }
        
        product >>= 1;
        term >>= 1;    
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