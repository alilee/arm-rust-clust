/// Exception handling and context switching between threads.
///
/// Requires TPIDRRO_EL0 to contain pointer to TCB where register state can be saved.
use super::device_tree;

pub mod gic;
mod timer;

use log::info;

fn tick(_irq: u32, duration: u64) {
    use cortex_a::regs::*;
    info!("tick!");
    CNTP_TVAL_EL0.set(duration as u32);
}

pub fn init() -> Result<(), u64> {
    info!("init");

    extern "C" {
        static vector_table_el1: u64;
    }
    unsafe {
        use cortex_a::regs::*;
        VBAR_EL1.set(&vector_table_el1 as *const u64 as u64);
    };

    let dtb = device_tree::get_dtb();
    gic::init(dtb)?;
    gic::reset();
    let timer_irq = timer::set(62500000 * 4, tick).unwrap();
    gic::enable_irq(timer_irq);

    unsafe {
        unmask_interrupts();
    }

    Ok(())
}

pub unsafe fn unmask_interrupts() {
    use cortex_a::regs::*;

    DAIF.modify(DAIF::I::CLEAR);
}

#[no_mangle]
#[naked]
fn el1_sp0_sync_handler() -> ! {
    use cortex_a::regs::*;

    info!("SP0 Sync Exception!");
    info!("SPSR_EL1: {:b}", SPSR_EL1.get());
    info!("ESR_EL1: {:b}", ESR_EL1.get());
    info!("ESR_EL1::EC {:b}", ESR_EL1.read(ESR_EL1::EC));
    info!(
        "{}",
        match ESR_EL1.read(ESR_EL1::EC) {
            0b010101 => "SVC64",
            0b111100 => "BRK instruction execution in AArch64 state",
            _ => "Unknown exception class",
        }
    );
    info!("ESR_EL1::IL {:b}", ESR_EL1.read(ESR_EL1::IL));
    info!("ESR_EL1::ISS {:b}", ESR_EL1.read(ESR_EL1::ISS));
    info!("FAR_EL1: {:p}", FAR_EL1.get() as *const ());
    info!("ELR_EL1: {:p}", ELR_EL1.get() as *const ());

    info!("looping...");
    loop {}
}

#[no_mangle]
#[naked]
fn el1_sp1_sync_handler() -> ! {
    use cortex_a::regs::*;

    info!("SP1 Sync Exception!");
    info!("SPSR_EL1: {:b}", SPSR_EL1.get());
    info!("ESR_EL1: {:b}", ESR_EL1.get());
    info!(
        "ESR_EL1::EC {:b} - {}",
        ESR_EL1.read(ESR_EL1::EC),
        match ESR_EL1.read(ESR_EL1::EC) {
            0b010101 => "SVC64",
            0b100001 => "Instruction Abort (from EL1)",
            0b100101 => "Data Abort (from EL1)",
            0b111100 => "BRK instruction execution in AArch64 state",
            _ => "Unknown exception class",
        }
    );
    info!("ESR_EL1::IL {:b}", ESR_EL1.read(ESR_EL1::IL));
    info!("ESR_EL1::ISS {:b}", ESR_EL1.read(ESR_EL1::ISS));
    info!("FAR_EL1: {:p}", FAR_EL1.get() as *const ());
    info!("ELR_EL1: {:p}", ELR_EL1.get() as *const ());

    info!("looping...");
    loop {}
}

#[no_mangle]
fn el0_64_sync_handler() -> () {
    use cortex_a::regs::*;

    info!("EL0 Synchronous Exception!");
    info!("SPSR_EL1: {:b}", SPSR_EL1.get());
    info!("ESR_EL1: {:b}", ESR_EL1.get());
    info!("ESR_EL1::EC {:b}", ESR_EL1.read(ESR_EL1::EC));
    info!(
        "{}",
        match ESR_EL1.read(ESR_EL1::EC) {
            0b010101 => "SVC64",
            0b011000 => "MSR, MRS, or System instruction execution",
            0b111100 => "BRK instruction execution in AArch64 state",
            _ => "Unknown exception class",
        }
    );
    info!("ESR_EL1::IL {:b}", ESR_EL1.read(ESR_EL1::IL));
    info!("ESR_EL1::ISS {:b}", ESR_EL1.read(ESR_EL1::ISS));
    info!("FAR_EL1: {:p}", FAR_EL1.get() as *const ());
    info!("ELR_EL1: {:p}", ELR_EL1.get() as *const ());

    gic::print_state();

    info!("DAIF: 0b{:b}", DAIF.get());
    info!("CNTP_TVAL_EL0: 0x{:x}", CNTP_TVAL_EL0.get());
    info!("CNTP_CTL_EL0: 0b{:b}", CNTP_CTL_EL0.get());

    info!("looping...");
    loop {}
}

#[no_mangle]
fn el0_64_irq_handler() -> () {
    info!("EL0 IRQ Exception!");
    let int = gic::ack_int();
    gic::dispatch(int);
    gic::end_int(int);
}

pub fn supervisor(syndrome: u16) -> () {
    match syndrome {
        99 => unsafe { asm!("svc 99" :::: "volatile") },
        _ => {}
    }
}

pub fn resume() -> ! {
    unsafe {
        asm!("b handler_return");
    }
    unreachable!()
}

//pub fn current_el() -> u32 {
//    use cortex_a::regs::*;
//    CurrentEL.get()
//}

#[inline(always)]
pub fn disable_irq() {
    unsafe { asm!("msr DAIFSet, $0"::"i"(1<<1)) }
}

#[inline(always)]
pub fn enable_irq() {
    unsafe { asm!("msr DAIFClr, $0"::"i"(1<<1)) }
}

global_asm!(
    r#"
    .section        .handler
    .global         vector_table_el1, handler_return

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
                    mrs     x2, tpidr_el1
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
                    adr     x30, handler_return
                    b       el0_64_sync_handler

    /* IRQ or vIRQ */
    .balign         0x80
    el0_64_irq:     stp     x2, x3, [sp, #-16]! // push x2, x3 onto EL1 stack
                    mrs     x2, tpidr_el1       // address of control block
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
                    adr     x30, handler_return
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
    handler_return: mrs     x30, tpidr_el1
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
