// SPDX-License-Identifier: Unlicense

//! Interaction with physical exceptions

use crate::pager::{Addr, VirtAddr};
use crate::Result;

use core::arch::asm;

use cortex_a::registers::{ESR_EL1, SPSR_EL1};

use tock_registers::{
    interfaces::{Readable, Writeable},
    LocalRegisterCopy,
};

pub fn set_vbar() -> Result<()> {
    use cortex_a::registers::*;

    extern "C" {
        static vector_table_el1: u8;
    }
    let p_vector_table = unsafe { VirtAddr::from_linker_symbol(&vector_table_el1).get() as u64 };

    VBAR_EL1.set(p_vector_table);

    Ok(())
}

pub type SpsrEL1 = LocalRegisterCopy<u64, SPSR_EL1::Register>;
pub type EsrEL1 = LocalRegisterCopy<u64, ESR_EL1::Register>;

/// The exception context as it is stored on the stack on exception entry.
#[repr(C)]
pub struct ExceptionContext {
    /// General Purpose Registers.
    gpr: [u64; 30],

    /// The link register, aka x30.
    lr: u64,

    /// Exception link register. The program counter at the time the exception happened.
    elr_el1: u64,

    /// Saved program status.
    spsr_el1: SpsrEL1,

    // Exception syndrome register.
    esr_el1: EsrEL1,
}

fn print_exception_details() {
    use cortex_a::registers::*;

    info!("SPSR_EL1: {:b}", SPSR_EL1.get());
    info!("ESR_EL1: {:b}", ESR_EL1.get());
    info!("ESR_EL1::EC {:b}", ESR_EL1.read(ESR_EL1::EC));
    // info!("{:?}", ESR_EL1.read_as_enum(ESR_EL1::EC));
    info!("ESR_EL1::IL {:b}", ESR_EL1.read(ESR_EL1::IL));
    info!("ESR_EL1::ISS {:b}", ESR_EL1.read(ESR_EL1::ISS));
    info!("FAR_EL1: {:p}", FAR_EL1.get() as *const ());
    info!("ELR_EL1: {:p}", ELR_EL1.get() as *const ());
}

#[allow(dead_code)]
#[no_mangle]
fn el1_sp0_sync_handler() -> ! {
    info!("SP0 Sync Exception!");
    print_exception_details();

    info!("looping...");
    loop {}
}

#[no_mangle]
fn el1_sp1_sync_handler(exc: &ExceptionContext) -> Option<u64> {
    use cortex_a::registers::{ESR_EL1::*, *};

    info!("SP1 Sync Exception!");
    print_exception_details();

    match exc.esr_el1.read_as_enum(ESR_EL1::EC) {
        Some(EC::Value::DataAbortCurrentEL) => {
            super::pager::handle_data_abort_current_el(exc.esr_el1)
        }
        Some(EC::Value::SVC64) => {
            handle_svc64(exc.esr_el1.read(ESR_EL1::ISS_SVC_IMMEDIATE) as u16, exc)
        }
        None => unreachable!(),
        _ => {
            panic!("unknown exception: 0b{:06b}", exc.esr_el1.read(ESR_EL1::EC));
        }
    }
}

///
pub fn handle_svc64(imm: u16, _exc: &ExceptionContext) -> Option<u64> {
    info!("handle_svc64");

    match imm {
        0 => None, // no-op
        _ => panic!("unknown SVC immediate: {:?}", imm),
    }
}

#[no_mangle]
fn el0_64_sync_handler() -> () {
    use cortex_a::registers::*;

    info!("EL0 Synchronous Exception!");
    print_exception_details();

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

#[no_mangle]
fn default_handler(_exc: &ExceptionContext) -> () {
    info!("default handler");
    print_exception_details();
}

#[no_mangle]
#[inline(always)]
extern "C" fn sleep_user_test(ms: u64) -> u64 {
    let result;
    unsafe { asm!("svc #1", in("x0") ms, lateout("x0") result, clobber_abi("C")) }
    result
}

#[no_mangle]
extern "C" fn sleep_kernel_test(ms: u64) -> u64 {
    unsafe { asm!("mov x22, #345", out("x22") _) }
    ms + 1
}

core::arch::global_asm!(
    r#"
.global             vector_table_el1

.macro              EXCEPTION_ENTRY handler
                    // Make room on the stack for the exception context.
                    sub	sp,  sp,  #16 * 17
            
                    // Store all general purpose registers on the stack.
                    stp	x0,  x1,  [sp, #16 * 0]
                    stp	x2,  x3,  [sp, #16 * 1]
                    stp	x4,  x5,  [sp, #16 * 2]
                    stp	x6,  x7,  [sp, #16 * 3]
                    stp	x8,  x9,  [sp, #16 * 4]
                    stp	x10, x11, [sp, #16 * 5]
                    stp	x12, x13, [sp, #16 * 6]
                    stp	x14, x15, [sp, #16 * 7]
                    stp	x16, x17, [sp, #16 * 8]
                    stp	x18, x19, [sp, #16 * 9]
                    stp	x20, x21, [sp, #16 * 10]
                    stp	x22, x23, [sp, #16 * 11]
                    stp	x24, x25, [sp, #16 * 12]
                    stp	x26, x27, [sp, #16 * 13]
                    stp	x28, x29, [sp, #16 * 14]
                
                    mrs	x1,  ELR_EL1        // exception link
                    mrs	x2,  SPSR_EL1       // saved program status
                    mrs	x3,  ESR_EL1        // exception syndrome
                
                    stp	lr,  x1,  [sp, #16 * 15]
                    stp	x2,  x3,  [sp, #16 * 16]
                
                    // pass context as first argument
                    mov	x0,  sp
                    adr lr, .handler_return
                    b	\handler
.endm

.balign 0x800       /* Exception taken from EL1 with SP_EL0. */
vector_table_el1:   mov x0, 0
				    EXCEPTION_ENTRY default_handler
.balign 0x080       /* IRQ or vIRQ */
                    mov x0, 1
				    EXCEPTION_ENTRY default_handler
.balign 0x080       /* FIQ or vFIQ */
				    mov     x0, 2
				    EXCEPTION_ENTRY default_handler
.balign 0x080       /* SError or vSError */
				    mov     x0, 3
				    EXCEPTION_ENTRY default_handler
				  
.balign 0x080       /* Exception taken from EL1 with SP_EL1. */
                    /* Synchronous */
				    EXCEPTION_ENTRY el1_sp1_sync_handler
.balign 0x080
				    mov     x0, 5
				    EXCEPTION_ENTRY default_handler
.balign 0x080
				    mov     x0, 6
				    EXCEPTION_ENTRY default_handler
.balign 0x080
				    mov     x0, 7
				    EXCEPTION_ENTRY default_handler
.balign 0x080
.handler_return:    ldr	w19,      [sp, #16 * 16]
                    ldp	lr,  x20, [sp, #16 * 15]
                    msr	SPSR_EL1, x19
                    msr	ELR_EL1,  x20
                
                    ldp	x0,  x1,  [sp, #16 * 0]
                    ldp	x2,  x3,  [sp, #16 * 1]
                    ldp	x4,  x5,  [sp, #16 * 2]
                    ldp	x6,  x7,  [sp, #16 * 3]
                    ldp	x8,  x9,  [sp, #16 * 4]
                    ldp	x10, x11, [sp, #16 * 5]
                    ldp	x12, x13, [sp, #16 * 6]
                    ldp	x14, x15, [sp, #16 * 7]
                    ldp	x16, x17, [sp, #16 * 8]
                    ldp	x18, x19, [sp, #16 * 9]
                    ldp	x20, x21, [sp, #16 * 10]
                    ldp	x22, x23, [sp, #16 * 11]
                    ldp	x24, x25, [sp, #16 * 12]
                    ldp	x26, x27, [sp, #16 * 13]
                    ldp	x28, x29, [sp, #16 * 14]
                
                    add	sp,  sp,  #16 * 17
                    eret
"#
);
