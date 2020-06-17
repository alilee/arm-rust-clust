// SPDX-License-Identifier: Unlicense

//! Interaction with physical exceptions

use crate::pager::{Addr, VirtAddr};
use crate::Result;

pub fn set_vbar() -> Result<()> {
    use cortex_a::regs::*;

    extern "C" {
        static vector_table_el1: u8;
    }
    let p_vector_table = unsafe { VirtAddr::from_linker_symbol(&vector_table_el1).get() as u64 };

    VBAR_EL1.set(p_vector_table);

    Ok(())
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
            0b100100 => "Data Abort from EL0",
            0b111100 => "BRK instruction execution in AArch64 state",
            _ => "Unknown exception class",
        }
    );
    info!("ESR_EL1::IL {:b}", ESR_EL1.read(ESR_EL1::IL));
    info!("ESR_EL1::ISS {:b}", ESR_EL1.read(ESR_EL1::ISS));
    info!("FAR_EL1: {:p}", FAR_EL1.get() as *const ());
    info!("ELR_EL1: {:p}", ELR_EL1.get() as *const ());

    // gic::print_state();

    info!("DAIF: 0b{:b}", DAIF.get());
    info!("CNTP_TVAL_EL0: 0x{:x}", CNTP_TVAL_EL0.get());
    info!("CNTP_CTL_EL0: 0b{:b}", CNTP_CTL_EL0.get());

    info!("looping...");
    loop {}
}

#[no_mangle]
fn el0_64_irq_handler() -> () {
    info!("EL0 IRQ Exception!");
    loop {}
    // let int = gic::ack_int();
    // gic::dispatch(int);
    // gic::end_int(int);
}

global_asm!(
    r#"
.global           vector_table_el1

.macro            EXCEPTION_ENTRY handler
				  stp     x2, x3, [sp, #-16]! // push x2, s3
				  mrs     x2, tpidr_el1
				  stp     x0, x1, [x2], #16   // save x0, x1
				  ldp     x0, x1, [sp], #16   // pop original x2, x3 into x0, x1
				  stp     x0, x1, [x2], #16   // save x2, x3
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
				  adr     x30, .handler_return
				  b       \handler
.endm

.balign 0x800     /* Exception taken from EL1 with SP_EL0. */
vector_table_el1: EXCEPTION_ENTRY el1_sp0_sync_handler
.balign 0x080     /* IRQ or vIRQ */
				  mov     x0, 1
				  adr     x30, .handler_return
				  b       .
.balign 0x080     /* FIQ or vFIQ */
				  mov     x0, 2
				  adr     x30, .handler_return
				  b       .
.balign 0x080     /* SError or vSError */
				  mov     x0, 3
				  adr     x30, .handler_return
				  b       .
.balign 0x080     /* Exception taken from EL1 with SP_EL1. */
                  /* Synchronous */
				  mov     x0, 4
				  bl      el1_sp1_sync_handler
				  b       .
.balign 0x080
				  mov     x0, 5
				  adr     x30, .handler_return
				  b       .
.balign 0x080
				  mov     x0, 6
				  adr     x30, .handler_return
				  b       .
.balign 0x080
				  mov     x0, 7
				  adr     x30, .handler_return
				  b       .
.balign 0x080
.handler_return:  mrs        x30, tpidr_el1
                  ldp        x0, x1, [x30], #16
				  ldp        x2, x3, [x30], #16
				  ldp        x4, x5, [x30], #16
				  ldp        x6, x7, [x30], #16
				  ldp        x8, x9, [x30], #16
				  ldp        x10, x11, [x30], #16
				  ldp        x12, x13, [x30], #16
				  ldp        x14, x15, [x30], #16
				  ldp        x16, x17, [x30], #16
				  ldp        x18, x19, [x30], #16
				  ldp        x20, x21, [x30], #16
				  ldp        x22, x23, [x30], #16
				  ldp        x24, x25, [x30], #16
				  ldp        x26, x27, [x30], #16
				  ldp        x28, x29, [x30], #16
				  ldr        x30, [x30]
				  eret
"#
);
