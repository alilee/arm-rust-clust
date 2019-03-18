/// Exception handling and context switching between threads.
///
/// Requires TPIDRRO_EL0 to contain pointer to TCB where register state can be saved.

pub mod gic;

use log::info;


pub fn init() {
    extern "C" {
        static vector_table_el1: u64;
    }
    unsafe {
        use cortex_a::regs::*;
        VBAR_EL1.set(&vector_table_el1 as *const u64 as u64);
    };

    gic::init();
}


#[no_mangle]
fn el1_sp0_sync_handler() -> () {
    info!("SP0 Sync Exception!");
}


#[no_mangle]
fn el1_sp1_sync_handler() -> () {
    info!("SP1 Sync Exception!");
}


#[no_mangle]
fn el0_64_sync_handler() -> () {
    info!("EL0 Synchronous Exception!");
    loop {}
}


enum IRQReason {
    TimerSlice
}

#[no_mangle]
fn el0_64_irq_handler() -> () {
    info!("EL0 IRQ Exception!");
    // what is reason
    let reason = IRQReason::TimerSlice;
    match reason {
        IRQReason::TimerSlice => crate::thread::yield_slice(),
        _ => crate::panic(),
    }
}


pub fn supervisor(syndrome: u16) -> () {
    use cortex_a::svc;
    match syndrome {
        99 => svc!(99),
        _ => {}
    }
}


global_asm!(
    r#"
    .section        .handler
    .global         vector_table_el1

    .balign         0x800
    vector_table_el1:

    /* Exception taken from EL1 with SP_EL0. */
    /* Synchronous */
    el1_sp0_sync:   mov     x0, xzr
                    b       .
    /* IRQ or vIRQ */
    .balign         0x80
    el1_sp0_irq:    mov     x0, xzr
                    b       .
    /* FIQ or vFIQ */
    .balign         0x80
    el1_sp0_fiq:    mov     x0, xzr
                    b       .
    /* SError or vSError */
    .balign         0x80
    el1_sp0_serror: mov     x0, xzr
                    b       .

    .balign         0x80
    /* Exception taken from EL1 with SP_EL1. */
    /* Synchronous */
    el1_sp1_sync:   stp     x0, x1, [sp, #-16]!


                    ldr     x30, handler_return
                    b el1_sp1_sync_handler

    /* IRQ or vIRQ */
    .balign         0x80
    el1_sp1_irq:    mov     x0, xzr
                    b       .
    /* FIQ or vFIQ */
    .balign         0x80
    el1_sp1_fiq:    mov     x0, xzr
                    b       .
    /* SError or vSError */
    .balign         0x80
    el1_sp1_serror: mov     x0, xzr
                    b       .

    .balign         0x80
    /* Exception taken from EL0/AArch64. */
    /* Synchronous */
    el0_64_sync:    stp     x2, x3, [sp, #-16]! // push x2, x3
                    mrs     x2, tpidrro_el0
                    stp     x0, x1, [x2], #16   // save x0, x1
                    ldp     x0, x1, [sp], #16   // pop original x2, x3 into x0, x1
                    stp     x0, x1, [x2], #16   // push x2, x3
                    stp     x4, x5, [x2], #16   // etc
                    stp     x6, x7, [x2], #16
                    stp     x8, x9, [x2], #16
                    stp     x10, x11, [x2], #16
                    stp     x12, x13, [x2], #16
                    stp     x14, x15, [x2], #16
                    stp     x16, x17, [x2], #16
                    stp     x18, x19, [x2], #16
                    stp     x20, x21, [x2], #16
                    stp     x22, x23, [x2], #16
                    stp     x24, x25, [x2], #16
                    stp     x26, x27, [x2], #16
                    stp     x28, x29, [x2], #16
                    stp     x30, xzr, [x2], #16
                    ldr     x30, handler_return
                    b       el0_64_sync_handler

    /* IRQ or vIRQ */
    .balign         0x80
    el0_64_irq:     stp     x2, x3, [sp, #-16]! // push x2, x3 onto EL1 stack
                    mrs     x2, tpidrro_el0     // address of control block
                    stp     x0, x1, [x2], #16   // save x0, x1
                    ldp     x0, x1, [sp], #16   // pop original x2, x3 into x0, x1
                    stp     x0, x1, [x2], #16   // push x2, x3
                    stp     x4, x5, [x2], #16   // etc
                    stp     x6, x7, [x2], #16
                    stp     x8, x9, [x2], #16
                    stp     x10, x11, [x2], #16
                    stp     x12, x13, [x2], #16
                    stp     x14, x15, [x2], #16
                    stp     x16, x17, [x2], #16
                    stp     x18, x19, [x2], #16
                    stp     x20, x21, [x2], #16
                    stp     x22, x23, [x2], #16
                    stp     x24, x25, [x2], #16
                    stp     x26, x27, [x2], #16
                    stp     x28, x29, [x2], #16
                    stp     x30, xzr, [x2]
                    ldr     x30, handler_return
                    b       el0_64_irq_handler
    /* FIQ or vFIQ */
    .balign         0x80
    el0_64_fiq:     mov     x0, xzr
                    b       .
    /* SError or vSError */
    .balign         0x80
    el0_64_serror:  mov     x0, xzr
                    b       .

    .balign         0x80
    /* Exception taken from EL0/AArch32. */
    /* Synchronous */
    el0_32_sync:    mov     x0, xzr
                    b       .
    /* IRQ or vIRQ */
    .balign         0x80
    el0_32_irq:     mov     x0, xzr
                    b       .
    /* FIQ or vFIQ */
    .balign         0x80
    el0_32_fiq:     mov     x0, xzr
                    b       .
    /* SError or vSError */
    .balign         0x80
    el0_32_serror:  mov     x0, xzr
                    b       .

    /* Return from handler */
    .balign         0x80
    handler_return: mrs     x30, tpidrro_el0
                    ldp     x0, x1, [x30], #16
                    ldp     x2, x3, [x30], #16
                    ldp     x4, x5, [x30], #16
                    ldp     x6, x7, [x30], #16
                    ldp     x8, x9, [x30], #16
                    ldp     x10, x11, [x30], #16
                    ldp     x12, x13, [x30], #16
                    ldp     x14, x15, [x30], #16
                    ldp     x16, x17, [x30], #16
                    ldp     x18, x19, [x30], #16
                    ldp     x20, x21, [x30], #16
                    ldp     x22, x23, [x30], #16
                    ldp     x24, x25, [x30], #16
                    ldp     x26, x27, [x30], #16
                    ldp     x28, x29, [x30], #16
                    ldr     x30, [x30]
                    eret

    "#
);
