/// Capabilities describe a thread's privileges to access cluster resources and consume services.
///
/// Subjects of capabilities, and rights:
///    Thread (create, yield, block, resume, send, debug, terminate)
///    Address range (reserve, release, read, write, execute, share)
///    System service (eg. cpu/board state?, service name->thread id resolution?)
///    Device/IORange (eg. network, gpu, keyboard, mouse, audio, gpio, storage, serial)
///
/// (thread, subject, rights)
/// (threadid=3, subject=AddrRange(0..0x10000000), rights=RW)
/// (threadid=4, subject=IORange(0xb80000, rights=W), rights=RW)
///
/// Who grants a thread capabilities? A thread who owns those capabilities.
///   On creation, by the creating thread
///   On request, by an owning thread
///
/// Requires trusted IPC to authenticate requester
///
/// Does subject pack into a u64?
///   AddrRange(63:56 -> ADDR, 55:12 -> page_base, 11:0 -> number of pages)
///   ThreadID(63:56 -> THRD, 55:0 -> id)
///   IORange(63:56 -> IORG, 55:32 -> NodeID, 31:8 -> page_base >> 12, 7:0 number of pages)
/// Are rights a u64, with one bit per right, according to subject type (max. 64 rights)?
use super::ThreadID;
use crate::pager;
use pager::virt_addr::{VirtAddr, VirtAddrRange};
use register::LocalRegisterCopy;

use core::collections::HashSet;

pub enum ThreadRight {
    Create,
    Yield,
    Block,
    Resume,
    Send,
    Debug,
    Terminate,
}

pub type ThreadRights = HashSet<ThreadRight>;

pub enum AddrRangeRight {
    Reserve,
    Release,
    Read,
    Write,
    Execute,
    Share,
}

pub type AddrRangeRights = HashSet<AddrRangeRight>;

impl From<pager::attrs::Attributes> for AddrRangeRights {
    fn from(attrs: pager::attrs::Attributes) -> Self {
        use pager::attrs::Attributes::AttributeFields::*;
        use AddrRangeRight::*;

        let result = HashSet::new();
        result.insert(Reserve, Release, Share);
        if attrs.is_set(UserRead) {
            result.insert(Read);
        }
        if attrs.is_set(UserWrite) {
            result.insert(Write);
        }
        if attrs.is_set(UserExec) {
            result.insert(Execute);
        }
        result
    }
}

pub enum Capability {
    Thread(ThreadID, ThreadRights),
    AddrRange(VirtAddrRange, AddrRangeRights),
}

impl From<Capability> for VirtAddrRange {
    fn from(cap: Capability) -> Self {
        if let Capability::AddrRange(virt_range, _) = cap {
            virt_range
        } else {
            VirtAddrRange::null()
        }
    }
}
