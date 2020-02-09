/// The thread module manages the set of local TCBs and scheduling.
///
/// A TCB is a space for thread state to be stored when not executing, to
/// pass messages for IPC.
///
///
///
pub mod cap;

use super::arch;
use crate::arch::thread::ControlBlock;
use crate::dbg;
use crate::pager;
use crate::range;
use crate::util::locked::Locked;
use pager::{layout, Page, PhysAddr};

use log::{debug, info, trace};

use core::panic;
use core::pin::Pin;
use crate::pager::attrs::kernel_read_write;
use crate::pager::virt_addr::VirtAddrRange;
use crate::pager::MemOffset;

#[derive(Copy, Clone, Debug)]
pub struct ThreadID(usize);

impl From<Pin<&Thread>> for ThreadID {
    fn from(thread: Pin<&Thread>) -> Self {
        let result = thread as *const Thread as usize;
        Self(result)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unused = 0,
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Clone, Copy, Debug)]
#[repr(C), align(4096)]
struct Thread {
    arch_tcb: arch::thread::ControlBlock,
    state: State,
    priority: i32,
}

static THREADS: Locked<HashSet<ThreadID>> = Locked::new(HashSet::new());

impl Thread {
    pub fn reset(&mut self, f: fn() -> (), stack_top: *const Page, user_tt_page: PhysAddr) -> Result<(), u64> {
        self.arch_tcb = arch::thread::ControlBlock::spawn(f, stack_top, user_tt_page);
        self.state = State::Blocked;
        self.priority = 0;
        assert_eq!(
            &result as *const Thread as *const ControlBlock,
            &result.arch_tcb as *const ControlBlock
        );
        let _x = masking_interrupts(|| THREADS.lock().insert(ThreadID::from(result)));
        result
    }

    pub fn spawn_into_page(
        page: *const Page,
        f: fn() -> (),
        stack_top: *const Page,
        user_tt_page: PhysAddr,
    ) -> Result<&mut Thread, u64> {
        debug!("spawn");
        let thread = unsafe { &mut (*(page as *mut Thread)) };
        thread.reset(f, stack_top, user_tt_page);
        Ok(thread)
    }

    /// Find the TCB for the currently running user thread
    pub fn current() -> &'static mut Thread {
        let current = arch::thread::ControlBlock::current();
        let pt = current as *mut arch::thread::ControlBlock as *mut Thread;
        trace!("{:?}", pt);
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
            find_state(&STATES[..], State::Ready)
                .map(|slot| {
                    STATES[slot] = State::Blocked;
                    slot
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

    pub fn thread_id(self: &Thread) -> ThreadID {
        ThreadID(self.slot)
    }

    pub fn state(self: &Thread) -> State {
        unsafe { STATES[self.slot] }
    }

    pub fn terminate(self: &mut Thread) -> () {
        trace!("terminate");
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
/// FIXME: Ensure that size_of<Thread> < 1 page
pub fn init() -> Result<(), u64> {
    info!("init");
    arch::thread::init();

    let attributes = kernel_read_write();
    let cap = range::reserve(1, false, attributes)?;
    let virt_range = VirtAddrRange::from(cap);
    let page = pager::allocate(virt_range, attributes)?;
    let stack_top = range::null(); // we will never drop to user
    let user_tt_page: PhysAddr = arch::pager::user_tt_page(); // populated through pager::init
    let thread = Thread::spawn_into(page, crate::user::thread::terminate, stack_top, user_tt_page)?;
    thread.arch_tcb.restore_cpu(); // this is now the running thread
    Ok(())
}

pub fn spawn(f: fn() -> ()) -> Result<ThreadID, u64> {
    let user_stack = range::reserve(1, true)?;
    let tt0 = pager::alloc_pool(1)?;
    let result = Thread::spawn(f, user_stack, tt0.base_mut())?;
    Ok(result.thread_id())
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
        None => panic!(),
    }
}

/// Declare a thread ready
pub fn ready(tid: ThreadID) -> () {
    info!("ready {:?}", tid);
    let t = Thread::find(tid);
    match t {
        Some(t) => t.ready(),
        None => panic!(),
    }
}

pub fn terminate() -> ! {
    info!("terminate");
    let current = Thread::current();
    debug!("terminating: {:?}", current);
    unsafe {
        spinlock::exclusive(&mut THREADS_SL, || {
            current.terminate();
            Thread::next_ready().map(|next| {
                next.running();
                next
            })
        })
        .map(|next| {
            next.arch_tcb.restore_cpu();
        });
    }
    arch::handler::resume()
}

#[inline(always)]
pub fn masking_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    use arch::handler::{are_interrupts_masked, mask_interrupts, unmask_interrupts};

    let were_masked = are_interrupts_masked();

    if !were_masked {
        mask_interrupts();
    }

    let ret = f();

    if !were_masked {
        unmask_interrupts();
    }

    // return the result of `f` to the caller
    ret
}
