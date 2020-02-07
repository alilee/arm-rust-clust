/// aarch64-specific thread lifecycle and context switching
///
/// Manages a thread control blocks (TCBs) - one per spawned thread.
/// Does not do context switching - that is handler.
/// State in a separate ringbuffer for simple round-robin scheduling.
/// Array and ringbuffer access behind spinlock.
/// EL0 register state is saved into TCB on exception.
use log::{info, trace};

pub mod spinlock;

use crate::pager::Page;

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

    pub fn spawn(f: fn() -> (), stack: *const Page) -> ControlBlock {
        let mut res = ControlBlock {
            regs: [0; 32],
            sp_el0: stack as *const u64,
            elr: f as *const (),
            spsr: 0,
        };
        // LR
        res.regs[30] = crate::user::thread::terminate as *const fn() -> () as u64;
        res
    }

    pub fn current() -> &'static mut ControlBlock {
        let ptcb = TPIDR_EL1.get() as *mut ControlBlock;
        unsafe { &mut (*ptcb) }
    }

    pub fn store_cpu(self: &mut ControlBlock) -> () {
        self.sp_el0 = SP_EL0.get() as *const u64;
        self.elr = ELR_EL1.get() as *const ();
        self.spsr = SPSR_EL1.get();
    }

    pub fn restore_cpu(self: &ControlBlock) -> () {
        trace!("restore_cpu &self {:?}", self as *const ControlBlock);
        SP_EL0.set(self.sp_el0 as u64);
        ELR_EL1.set(self.elr as u64);
        SPSR_EL1.set(self.spsr);
        TPIDR_EL1.set(self as *const ControlBlock as u64);
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
