
pub mod pager;
pub mod handler;

pub fn drop_to_userspace() {
    // we need a stack
    unsafe {
        asm!("      adr x0, foo
                    msr elr_el1, x0
                    eret
              foo:  nop" ::: "x0");
    }
}
