
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

global_asm!(
    r#"
    .section        .startup
    .global         _reset

    _reset:         mrs     x7, CurrentEL
                    mrs     x6, CPACR_EL1
                    orr     x6, x6, 0x100000
                    msr     CPACR_EL1, x6

        	        ldr     x11, stack_top
        	        mov     sp, x11

                    ldr     x10, boot2
                    br      x10

                    b       .
    "#
);
