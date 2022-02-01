// SPDX-License-Identifier: Unlicense

//! A static array of nodes which are all in one of a set of deques.
//!
//! Provides O(1) for operations including:
//! - remove an item at an arbitrary index and push it onto a specific
//!   deque.
//! - remove an arbitrary sublist between two indices and push it onto
//!   a specific deque.
//! - pop an item from the back of one deque and push it to the top
//!   of another deque.
//! - move the entire sublist from one deque to the top of another.
//!
//! This doesn't have O(1) deque length because removals
//! can happen without knowing which deque the item is on.
//!
//! Removing m items from the tail of a deque is O(m) because the list
//! has to be traversed.
//!
//! The earliest entries in the array hold data, while additional nodes
//! at the back of the array are headers for each list. The prev and
//! next array indices in each node form circular queues.

use crate::{Error, Result};

use core::fmt::{Debug, Formatter};
use core::intrinsics::unchecked_sub;
use core::marker::PhantomData;
use core::mem::variant_count;
use core::ops::{Index, IndexMut};

struct DequeNode<N> {
    prev: u32,
    next: u32,
    node: N,
}

impl<N> Debug for DequeNode<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "prev: {}, next: {}", self.prev, self.next)
    }
}

impl<N: Default> DequeNode<N> {
    fn init_queue_empty(i: u32) -> Self {
        Self {
            prev: i,
            next: i,
            node: N::default(),
        }
    }

    fn init_entry(i: u32) -> Self {
        Self {
            prev: unsafe { unchecked_sub(i, 1) },
            next: i + 1,
            node: N::default(),
        }
    }

    #[allow(dead_code)]
    fn set_queue_empty(&mut self, q: u32) -> () {
        self.prev = q;
        self.next = q;
    }

    fn set_queue_seq(&mut self, i: u32, j: u32) -> () {
        self.prev = j;
        self.next = i;
    }
}

pub struct Deque<N: 'static, Q> {
    table: &'static mut [DequeNode<N>],
    _queues: PhantomData<Q>,
}

impl<N: Debug, Q: From<u8> + Debug> Debug for Deque<N, Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let queues = variant_count::<Q>();
        writeln!(f, "Deque: {{")?;
        for q in (self.table.len() - queues)..self.table.len() {
            let q_pos = (q - (self.table.len() - variant_count::<Q>())) as u8;
            let q_name: Q = q_pos.into();
            write!(f, "      {:?}: ", q_name)?;
            let mut i = self.table[q].next;
            while i != q as u32 {
                write!(f, "{}", i)?;
                let mut j = self.table[i as usize].next;
                let mut last = i;
                while j != q as u32 && j == (last + 1) {
                    last = j;
                    j = self.table[j as usize].next;
                }
                if j == q as u32 || last == i {
                    if j == q as u32 && last != i {
                        write!(f, "..{}, ", last)?;
                    } else {
                        write!(f, ", ")?;
                    }
                } else {
                    write!(f, "..{}, ", last)?;
                }
                i = j;
            }
            writeln!(f)?;
        }
        write!(f, "}}")
    }
}

impl<N: Default + Debug, Q: Copy + Into<u8> + From<u8> + Debug> Deque<N, Q> {
    pub fn storage_bytes(entries: usize) -> usize {
        (entries + variant_count::<Q>()) * core::mem::size_of::<DequeNode<N>>()
    }

    pub fn new(ptr: *mut u8, capacity: u32, init_q: Q) -> Self {
        let capacity = capacity as usize;
        let len = capacity + variant_count::<Q>();
        let table = unsafe { core::slice::from_raw_parts_mut(ptr as *mut DequeNode<N>, len) };
        for i in capacity..len {
            table[i] = DequeNode::<N>::init_queue_empty(i as u32);
        }
        for i in 0..capacity {
            table[i] = DequeNode::<N>::init_entry(i as u32);
        }
        let capacity = capacity as u32;
        let q: u32 = init_q.into() as u32;
        let q = capacity + q;
        table[q as usize].set_queue_seq(0, capacity - 1);
        table[0].prev = q;
        table[capacity as usize - 1].next = q;
        Self {
            table,
            _queues: Default::default(),
        }
    }

    pub fn repoint_table(&mut self, ptr: *mut u8, capacity: u32) -> Result<()> {
        let capacity = capacity as usize;
        let len = capacity + variant_count::<Q>();
        self.table = unsafe { core::slice::from_raw_parts_mut(ptr as *mut DequeNode<N>, len) };
        Ok(())
    }

    fn queue_entry(&self, q: Q) -> usize {
        let q = q.into() as usize;
        let len = self.table.len();
        let q_count = variant_count::<Q>();
        len - (q_count - q)
    }

    fn detach_seq(&mut self, i: usize, j: usize) -> () {
        info!("detach_seq: {}, {}", i, j);
        assert_lt!(i, self.top());
        assert_lt!(j, self.top());
        let prev = self.table[i].prev;
        let next = self.table[j].next;
        dbg!(prev);
        dbg!(next);
        self.table[prev as usize].next = next;
        self.table[next as usize].prev = prev;
    }

    fn push_seq(&mut self, i: usize, j: usize, q: usize) {
        assert_lt!(i, self.top());
        assert_lt!(j, self.top());
        let old_head = self.table[q].next as usize;
        self.table[q].next = i as u32;
        self.table[i].prev = q as u32;
        self.table[old_head].prev = j as u32;
        self.table[j].next = old_head as u32;
    }

    fn tail(&self, q: Q) -> usize {
        let q = q.into() as usize;
        let q = self.table.len() - (variant_count::<Q>() - q);
        dbg!(q);
        dbg!(&self.table[q]);
        self.table[q].prev as usize
    }

    #[allow(dead_code)]
    fn seq(&self, q: Q) -> (usize, usize) {
        let q = self.queue_entry(q);
        (self.table[q].next as usize, self.table[q].prev as usize)
    }

    fn top(&self) -> usize {
        self.table.len() - variant_count::<Q>()
    }

    #[cfg(test)]
    fn is_empty(&self, q: Q) -> bool {
        let q = self.queue_entry(q);
        self.table[q].next == self.table[q].prev
    }

    pub fn remove_to(&mut self, i: u32, q: Q) -> Result<u32> {
        assert_lt!(i as usize, self.top());
        self.remove_seq_to(i, i, q).map(|(i, _)| i)
    }

    pub fn remove_seq_to(&mut self, i: u32, j: u32, q: Q) -> Result<(u32, u32)> {
        info!("remove_seq_to: {}, {}, {:?}", i, j, q);
        dbg!(self.top());
        let i = i as usize;
        let j = j as usize;
        assert_lt!(i, self.top());
        assert_lt!(j, self.top());
        self.detach_seq(i, j);
        self.push_seq(i, j, self.queue_entry(q));
        Ok((i as u32, j as u32))
    }

    pub fn drip_to(&mut self, q_from: Q, q_to: Q) -> Result<u32> {
        info!("drip_to: {:?} -> {:?}", q_from, q_to);
        let result = self.drip_n_to(q_from, 1, q_to).map(|(i, _)| i);
        dbg!(&result);
        result
    }

    pub fn drip_n_to(&mut self, q_from: Q, mut n: u32, q_to: Q) -> Result<(u32, u32)> {
        major!("drip_n_to: {:?}/{} -> {:?}", q_from, n, q_to);
        let j = self.tail(q_from) as u32;
        dbg!(j);
        let mut i = j as usize;
        loop {
            if i == self.queue_entry(q_from) as usize {
                return Err(Error::OutOfPages);
            }
            if n == 1 {
                break;
            }
            i = self.table[i].prev as usize;
            n -= 1;
        }
        dbg!(i);
        dbg!(j);
        self.remove_seq_to(i as u32, j, q_to)
    }

    pub fn clear_to(&mut self, q_from: Q, q_to: Q) -> Result<(u32, u32)> {
        let (i, j) = self.seq(q_from);
        let q_from = self.queue_entry(q_from);
        if i == q_from {
            return Err(Error::Success);
        };
        self.table[q_from].set_queue_empty(q_from as u32);
        self.push_seq(i, j, self.queue_entry(q_to));
        Ok((i as u32, j as u32))
    }

    #[cfg(test)]
    fn count(&self, q: usize) -> u32 {
        let mut i = self.table[q].next as usize;
        let mut result = 0u32;
        while i != q {
            result += 1;
            i = self.table[i].next as usize;
        }
        result
    }

    #[cfg(test)]
    fn invariant(&self) -> bool {
        // sum(counts) == capacity
        let mut total_count = 0;
        dbg!(self.top());
        for q in self.top()..(self.top() + variant_count::<Q>()) {
            let count = self.count(q);
            dbg!((q, count));
            total_count += count
        }
        total_count == self.top() as u32
    }
}

impl<N: Default + Debug, Q: Copy + Into<u8> + From<u8> + Debug> Index<u32> for Deque<N, Q> {
    type Output = N;

    fn index(&self, index: u32) -> &Self::Output {
        assert_lt!(index as usize, self.top());
        &self.table[index as usize].node
    }
}

impl<N: Default + Debug, Q: Copy + Into<u8> + From<u8> + Debug> IndexMut<u32> for Deque<N, Q> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        assert_lt!(index as usize, self.top());
        &mut self.table[index as usize].node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::alloc::{alloc, dealloc, Layout};
    use core::mem;

    #[derive(Clone, Copy, Debug)]
    #[repr(u8)]
    enum Q {
        Free,
        Kernel,
        UserHot,
        UserWarm,
        UserCold,
    }

    impl From<u8> for Q {
        fn from(i: u8) -> Self {
            match i {
                i if i == Q::Free as u8 => Q::Free,
                i if i == Q::Kernel as u8 => Q::Kernel,
                i if i == Q::UserHot as u8 => Q::UserHot,
                i if i == Q::UserWarm as u8 => Q::UserWarm,
                i if i == Q::UserCold as u8 => Q::UserCold,
                _ => unreachable!(),
            }
        }
    }

    impl Into<u8> for Q {
        fn into(self) -> u8 {
            self as u8
        }
    }

    fn with_deque(capacity: u32, f: fn(&mut Deque<u64, Q>) -> ()) {
        unsafe {
            let len = capacity + variant_count::<Q>() as u32;
            dbg!(len);
            let layout =
                Layout::from_size_align(len as usize * mem::size_of::<DequeNode<u64>>(), 4096)
                    .unwrap();
            dbg!(layout);
            let ptr = alloc(layout);
            let mut d = Deque::<u64, _>::new(ptr, capacity, Q::Free);
            f(&mut d);
            dealloc(ptr, layout);
        }
    }

    #[test]
    fn test_init() {
        with_deque(10, |d| {
            dbg!(&d);
            assert!(d.invariant());
            assert!(!d.is_empty(Q::Free));
            assert!(d.is_empty(Q::Kernel));
            assert!(d.is_empty(Q::UserHot));
            assert!(d.is_empty(Q::UserWarm));
            assert!(d.is_empty(Q::UserCold));
        });
    }

    #[test]
    fn test_remove_seq_to() {
        with_deque(10, |d| {
            d.remove_seq_to(3, 3, Q::Kernel).unwrap();
            d.remove_seq_to(4, 5, Q::Kernel).unwrap();
            d.remove_seq_to(7, 9, Q::UserHot).unwrap();
            d.remove_seq_to(0, 0, Q::Kernel).unwrap();
            dbg!(&d);
            assert!(d.invariant());
        });
    }

    #[test]
    fn test_remove_to() {
        with_deque(10, |d| {
            d.remove_to(3, Q::Kernel).unwrap();
            d.remove_to(4, Q::UserHot).unwrap();
            d.remove_to(6, Q::UserWarm).unwrap();
            d.remove_to(0, Q::UserCold).unwrap();
            d.remove_to(9, Q::Kernel).unwrap();
            dbg!(&d);
            assert!(d.invariant());
        });
    }

    #[test]
    fn test_drip_to() {
        with_deque(10, |d| {
            d.remove_seq_to(3, 5, Q::Kernel).unwrap();
            d.remove_seq_to(0, 1, Q::UserWarm).unwrap();
            d.remove_seq_to(7, 8, Q::UserWarm).unwrap();
            d.remove_to(1, Q::UserHot).unwrap();
            d.remove_to(7, Q::UserHot).unwrap();
            let result = d.clear_to(Q::UserHot, Q::UserWarm).unwrap();
            assert_eq!(result, (7, 1));
            d.drip_to(Q::UserWarm, Q::UserCold).unwrap();
            d.drip_to(Q::UserWarm, Q::UserCold).unwrap();
            d.drip_n_to(Q::UserCold, 2, Q::Free).unwrap();
            assert!(d.is_empty(Q::UserCold));
            assert_err!(d.drip_to(Q::UserCold, Q::Free));
            dbg!(&d);
            assert!(d.invariant());
        });
    }

    #[test]
    fn test_clear_to() {
        with_deque(10, |d| {
            d.clear_to(Q::Free, Q::UserWarm).unwrap();
            d.drip_to(Q::UserWarm, Q::UserHot).unwrap();
            d.clear_to(Q::UserHot, Q::UserWarm).unwrap();
            assert_err!(d.clear_to(Q::UserHot, Q::UserWarm));
            dbg!(&d);
            assert!(d.invariant());
        });
    }
}
