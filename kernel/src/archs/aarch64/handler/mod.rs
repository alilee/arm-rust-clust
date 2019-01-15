

use log::info;


pub fn init() {
    extern "C" {
        static vector_table_el1: u64;
    }
    unsafe {
        use cortex_a::regs::*;
        VBAR_EL1.set(&vector_table_el1 as *const u64 as u64);
    };
}


#[no_mangle]
fn el1_sp0_sync_handler() -> () {
    info!("SP0 Sync Exception!");
}


#[no_mangle]
fn el1_sp1_sync_handler() -> () {
    info!("SP1 Sync Exception!");
}

#[inline]
pub fn supervisor() -> () {
    use cortex_a::svc;
    svc!(0);
}


global_asm!(
    r#"
    .section        .handler
    .global         vector_table_el1
    .extern         sp0_sync_handler

    .balign         0x800
    vector_table_el1:

    /* Exception taken from EL1 with SP_EL0. */
    /* Synchronous */
    el1_sp0_sync:   stp     x0, x1, [sp, #-16]!
                    stp     x2, x3, [sp, #-16]!
                    stp     x4, x5, [sp, #-16]!
                    stp     x6, x7, [sp, #-16]!
                    stp     x8, x9, [sp, #-16]!
                    stp     x10, x11, [sp, #-16]!
                    stp     x12, x13, [sp, #-16]!
                    stp     x14, x15, [sp, #-16]!
                    stp     x16, x17, [sp, #-16]!
                    stp     x18, x19, [sp, #-16]!
                    stp     x20, x21, [sp, #-16]!
                    stp     x22, x23, [sp, #-16]!
                    stp     x24, x25, [sp, #-16]!
                    stp     x26, x27, [sp, #-16]!
                    stp     x28, x29, [sp, #-16]!
                    stp     x30, x31, [sp, #-16]!
                    ldr     x30, handler_return
                    b       el1_sp0_sync_handler
    /* IRQ or vIRQ */
    .balign         0x80
    el1_sp0_irq:    ldr     x0, 0
                    b       .
    /* FIQ or vFIQ */
    .balign         0x80
    el1_sp0_fiq:    ldr     x0, 0
                    b       .
    /* SError or vSError */
    .balign         0x80
    el1_sp0_serror: ldr     x0, 0
                    b       .

    .balign         0x80
    /* Exception taken from EL1 with SP_EL1. */
    /* Synchronous */
    el1_sp1_sync:   stp     x0, x1, [sp, #-16]!
                    stp     x2, x3, [sp, #-16]!
                    stp     x4, x5, [sp, #-16]!
                    stp     x6, x7, [sp, #-16]!
                    stp     x8, x9, [sp, #-16]!
                    stp     x10, x11, [sp, #-16]!
                    stp     x12, x13, [sp, #-16]!
                    stp     x14, x15, [sp, #-16]!
                    stp     x16, x17, [sp, #-16]!
                    stp     x18, x19, [sp, #-16]!
                    stp     x20, x21, [sp, #-16]!
                    stp     x22, x23, [sp, #-16]!
                    stp     x24, x25, [sp, #-16]!
                    stp     x26, x27, [sp, #-16]!
                    stp     x28, x29, [sp, #-16]!
                    stp     x30, x31, [sp, #-16]!
                    ldr     x30, handler_return
                    b       el1_sp1_sync_handler
    /* IRQ or vIRQ */
    .balign         0x80
    el1_sp1_irq:    ldr     x0, 0
                    b       .
    /* FIQ or vFIQ */
    .balign         0x80
    el1_sp1_fiq:    ldr     x0, 0
                    b       .
    /* SError or vSError */
    .balign         0x80
    el1_sp1_serror: ldr     x0, 0
                    b       .

    .balign         0x80
    /* Exception taken from EL0/AArch64. */
    /* Synchronous */
    el0_64_sync:    ldr     x0, 0
                    b       .
    /* IRQ or vIRQ */
    .balign         0x80
    el0_64_irq:     ldr     x0, 0
                    b       .
    /* FIQ or vFIQ */
    .balign         0x80
    el0_64_fiq:     ldr     x0, 0
                    b       .
    /* SError or vSError */
    .balign         0x80
    el0_64_serror:  ldr     x0, 0
                    b       .

    .balign         0x80
    /* Exception taken from EL0/AArch32. */
    /* Synchronous */
    el0_32_sync:    ldr     x0, 0
                    b       .
    /* IRQ or vIRQ */
    .balign         0x80
    el0_32_irq:     ldr     x0, 0
                    b       .
    /* FIQ or vFIQ */
    .balign         0x80
    el0_32_fiq:     ldr     x0, 0
                    b       .
    /* SError or vSError */
    .balign         0x80
    el0_32_serror:  ldr     x0, 0
                    b       .

    /* Return from handler */
    .balign         0x80
    handler_return: ldp     x30, xzr, [sp], #16
                    ldp     x28, x29, [sp], #16
                    ldp     x26, x27, [sp], #16
                    ldp     x24, x25, [sp], #16
                    ldp     x22, x23, [sp], #16
                    ldp     x20, x21, [sp], #16
                    ldp     x18, x19, [sp], #16
                    ldp     x16, x17, [sp], #16
                    ldp     x14, x15, [sp], #16
                    ldp     x12, x13, [sp], #16
                    ldp     x10, x11, [sp], #16
                    ldp     x8, x9, [sp], #16
                    ldp     x6, x7, [sp], #16
                    ldp     x4, x5, [sp], #16
                    ldp     x2, x3, [sp], #16
                    ldp     x0, x1, [sp], #16
                    eret

    "#
);
