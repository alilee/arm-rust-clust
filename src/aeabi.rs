
#[no_mangle]
pub unsafe extern fn __aeabi_memclr4(s: *mut u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.offset(i as isize) = 0 as u8;
        i += 1;
    }
    return s;
}
