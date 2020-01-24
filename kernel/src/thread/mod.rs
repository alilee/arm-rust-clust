/// The thread module manages the set of local TCBs and scheduling.
///
/// A TCB is a space for thread state to be stored when not executing, to
/// pass messages for IPC.
///
///
///
use log::info;

use core::sync::atomic::AtomicBool;

use super::arch;
use crate::dbg;
use arch::thread::spinlock;

pub struct ThreadID(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unused = 0,
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Clone, Copy, Debug)]
pub struct Thread {
    arch_tcb: arch::thread::ControlBlock,
    slot: usize,
    priority: i32,
}

const MAX_THREADS: usize = 4;

static mut STATES: [State; MAX_THREADS] = [State::Unused; MAX_THREADS];
static mut THREADS: [Thread; MAX_THREADS] = [Thread {
    arch_tcb: arch::thread::ControlBlock::new(),
    slot: 0,
    priority: 0,
}; MAX_THREADS];

static mut THREADS_SL: AtomicBool = spinlock::new();

impl Thread {
    fn get_states() -> &'static [State] {
        unsafe { &STATES[..] }
    }

    pub fn spawn(f: fn() -> ()) -> Result<&'static mut Thread, u64> {
        info!("current state of STATES: {:?}", Thread::get_states());
        unsafe {
            let maybe_slot: Option<usize> = spinlock::exclusive(&mut THREADS_SL, || {
                find_state(&STATES[..], State::Unused).map(|slot| {
                    STATES[slot] = State::Blocked;
                    slot
                })
            });

            info!("candidate slot: {:?}", maybe_slot);

            maybe_slot
                .map(|slot| {
                    THREADS[slot] = Thread {
                        arch_tcb: arch::thread::ControlBlock::spawn(f),
                        slot: slot,
                        priority: 0,
                    };
                    &mut THREADS[slot]
                })
                .ok_or(0)
        }
    }

    /// Find the TCB for the currently running user thread
    pub fn current() -> &'static mut Thread {
        let current = arch::thread::ControlBlock::current();
        let pt = current as *mut arch::thread::ControlBlock as *mut Thread;
        unsafe { &mut (*pt) }
    }

    /// Find the next thread which is ready to execute.
    ///
    /// We assume that the previously running thread is still marked running or
    /// blocked so that it won't be selected. When selected, we block the thread
    /// so it can't be picked up by another core.
    ///
    /// TODO: Make this fair, rather than favouring the lower slots.
    pub fn next_ready() -> Option<&'static mut Thread> {
        unsafe {
            spinlock::exclusive(&mut THREADS_SL, || {
                find_state(&STATES[..], State::Ready).map(|slot| {
                    STATES[slot] = State::Blocked;
                    slot
                })
            })
            .map(|slot| &mut THREADS[slot])
        }
    }

    /// Return the TCB for the given ThreadID
    ///
    /// TODO: Error if ThreadID is unused
    pub fn find(tid: ThreadID) -> Option<&'static mut Thread> {
        let slot = tid.0;
        unsafe { Some(&mut THREADS[slot]) }
    }

    pub fn set_stack(self: &mut Thread, stack: &[u64]) -> () {
        self.arch_tcb.set_user_stack(stack);
    }

    pub fn thread_id(self: &Thread) -> ThreadID {
        ThreadID(self.slot)
    }

    pub fn state(self: &Thread) -> State {
        unsafe { STATES[self.slot] }
    }
    pub fn terminate(self: &mut Thread) -> () {
        unsafe {
            STATES[self.slot] = State::Terminated;
        }
    }
    pub fn ready(self: &mut Thread) -> () {
        unsafe {
            STATES[self.slot] = State::Ready;
        }
    }
    pub fn block(self: &mut Thread) -> () {
        unsafe {
            STATES[self.slot] = State::Blocked;
        }
    }
    pub fn running(self: &mut Thread) -> () {
        unsafe {
            STATES[self.slot] = State::Running;
        }
    }
    pub fn unused(self: &mut Thread) -> () {
        unsafe {
            STATES[self.slot] = State::Unused;
        }
    }
}

fn find_state(states: &[State], state: State) -> Option<usize> {
    states.iter().position(|s| *s == state)
}

/// Initialise the thread system on boot.
///
/// Assimilates the boot2 thread as already running. The boot thread never
/// had a user-space, but we establish the structures for one so that the
/// thread can be cleaned up normally.
/// The thread would behave as if it called the supervisor exception.
/// TODO: Instead, just find work as when thread yields.
pub fn init() -> () {
    fn terminator() {
        crate::user::thread::terminate();
    }

    info!("init");
    arch::thread::init();

    let slot = 0;
    unsafe {
        let arch_tcb = arch::thread::ControlBlock::spawn(terminator);
        STATES[slot] = State::Blocked;
        THREADS[slot] = Thread {
            arch_tcb,
            slot: 0,
            priority: 0,
        };
    }
}

/// Dump the state of the thread records for debugging.
pub fn show_state() {
    unsafe {
        dbg!(STATES);
        dbg!(THREADS);
    }
}

/// Select next thread and context switch.
pub fn yield_slice() -> () {
    let current = Thread::current();
    unsafe {
        spinlock::exclusive(&mut THREADS_SL, || {
            Thread::next_ready().map(|next| {
                current.arch_tcb.store_cpu();
                current.ready();
                next.running();
                next
            })
        })
        .map(|next| {
            next.arch_tcb.restore_cpu();
        });
    }
}

pub fn resume(tid: ThreadID) -> ! {
    let t = Thread::find(tid);
    match t {
        Some(t) => t.arch_tcb.resume(),
        None => crate::panic(),
    }
}

/// Declare a thread ready
pub fn ready(tid: ThreadID) -> () {
    let t = Thread::find(tid);
    match t {
        Some(t) => t.ready(),
        None => crate::panic(),
    }
}
