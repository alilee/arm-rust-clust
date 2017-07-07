
pub fn svc(a: u16) {
    unsafe {
        asm!("svc $0"::"i"(a));
    }
}

pub fn drop_to_userspace() {
    // we need a stack
    unsafe {
        asm!("     adr x0, foo
                   msr elr_el1, x0
                   eret
              foo: nop" ::: "x0");
    }
}
