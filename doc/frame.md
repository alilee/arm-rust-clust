counted, doubly linked list of free pages (ready for allocation, after zeroing out)
counted, doubly linked list of zero'd pages (ready for allocation)

- O(1) (push) on releasing page
- O(1) (pop) on alloc page

Idle tasks:
- zero'er: move pages from FREED to ZERO'd 
  - lt 5% of pages zero'd => priority high
  - lt 10% of pages zero'd => priority normal
  - _ if FREED empty => sleep 
  - _ => priority idle 

- cleaner: make dirty pages clean by updating backing store
  - gt 50% of cold pages dirty => priority normal
  - _ => priority idle

- evicter: select and evict non-dirty pages
  - lt 1% of pages either zero'd or free'd => priority high
  - lt 2% of pages either zero'd or free'd => priority normal
  - lt 5% of pages either zero'd or free'd => priority idle
  - _ => sleeping
  
- lru monitor: record accessed pages
  - each 5s: priority high
    - walk resident page tables (kernel's plus each thread's TT)
    - if is_set(AF): Hot::SET + AF::CLEAR
    - if !is_set(AF) && is_set(HotMap): Hot::CLEAR + WarmSet::Add + AF::CLEAR
  - each 20s: priority high
    - walk the warm set cohorts: if is_set(Hot): WarmSet::Remove 
    - if WarmSet gt 50% of pages: WarmSet::CLEAR + ColdSet::Add
    - walk the cold set: if is_set(Hot): ColdSet::Remove
    
    5s: if is_set(AF) { Hot::SET }
    

```rust
#[repr(align(4096))]
struct FreePage {
    previous: Option<PhysAddr>,
    next: Option<PhysAddr>,
}

static ZEROED: (usize, Option<PhysAddr>) = (99, Some(PhysAddr::new_const(0x40000000)));
static FREED: (usize, Option<PhysAddr>) = (0, None); 
```

 