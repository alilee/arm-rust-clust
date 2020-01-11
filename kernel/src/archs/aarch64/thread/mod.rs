/// aarch64-specific thread lifecycle and context switching
///
/// Manages a thread control blocks (TCBs) - one per spawned thread.
/// Does not do context switching - that is handler.
/// State in a separate ringbuffer for simple round-robin scheduling.
/// Array and ringbuffer access behind spinlock.
/// EL0 register state is saved into TCB on exception.
use log::info;

pub mod spinlock;

use core::ptr;
use cortex_a::regs::*;

#[derive(Clone, Copy, Debug)]
pub struct ControlBlock {
    regs: [u64; 32],    // save registers here on interrupt
    sp_el0: *const u64, // saved userspace stack pointer
    elr: *const (),     // saved EL0 exception resume address
    spsr: u32,          // saved processor flags state
}

impl ControlBlock {
    pub const fn new() -> ControlBlock {
        ControlBlock {
            regs: [0; 32],
            sp_el0: ptr::null(),
            elr: ptr::null(),
            spsr: 0,
        }
    }

    pub fn spawn(f: fn() -> ()) -> ControlBlock {
        let mut res = ControlBlock {
            regs: [0; 32],
            sp_el0: ptr::null(),
            elr: f as *const (),
            spsr: 0,
        };
        // LR
        res.regs[30] = crate::user::thread::terminate as *const fn() -> () as u64;
        res
    }

    pub fn current() -> &'static mut ControlBlock {
        let ptcb = TPIDRRO_EL0.get() as *mut ControlBlock;
        unsafe { &mut (*ptcb) }
    }

    pub fn set_user_stack(self: &mut ControlBlock, stack: &[u64]) -> () {
        const U64_SIZE: u64 = 8;
        let tos = &stack[stack.len() - 1] as *const u64 as u64 + U64_SIZE;
        self.sp_el0 = tos as *const u64;
    }

    pub fn store_cpu(self: &mut ControlBlock) -> () {
        self.sp_el0 = SP_EL0.get() as *const u64;
        self.elr = ELR_EL1.get() as *const ();
        self.spsr = SPSR_EL1.get();
    }

    pub fn restore_cpu(self: &ControlBlock) -> () {
        SP_EL0.set(self.sp_el0 as u64);
        ELR_EL1.set(self.elr as u64);
        SPSR_EL1.set(self.spsr);
        TPIDRRO_EL0.set(self as *const ControlBlock as u64);
    }

    pub fn resume(self: &ControlBlock) -> ! {
        self.restore_cpu();
        unsafe {
            asm!("b handler_return");
        }
        unreachable!()
    }
}

pub fn init() -> () {
    info!("init");
}
