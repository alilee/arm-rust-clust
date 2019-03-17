
# thesis
1.  One shared view of virtual address space.

2.  Local per-node view of page translation
    A VA will map to different physical pages on different nodes.
    No concept of process addressing (all pointers just work)

3.  Per node view of frames:
    The organisation of its physical memory is managed by the node.
    VM faults need to be resolved from local backing store or across the cluster.
    Other copies of shared memory need to be invalidated before then can be written to.
    All threads have the same view of all 4k pages, ie. they share the bottom layer of PTEs
    but different threads may not have the same 2MB ranges mapped.

# state of local ownership of a page
    (The state is recorded in our PTD)
    Valid-Dirty-Exclusive: Modified - I am the only node with a copy, memory only
    Valid-Clean-Exclusive: Exclusive - I am the only node with a copy, matches disk
    Valid-Dirty-Shared-Master: Shared Mem - I am not only node with a copy, memory only
    Valid-Dirty-Shared-Copy: Shared Mem - I am not only node with a copy, memory only
    Valid-Clean-Shared-Master: Master Disk - Other nodes may have copies, but I hold the master on disk
    Valid-Clean-Shared-Copy: Copy Disk - Another node has the master, but I have copy on disk
    Invalid-Exclusive: Exclusive Disk - I am the only node with a copy, only on disk
    Invalid-Shared-Master: Shared Disk Master - I have a critical copy to read from disk
    Invalid-Shared-Copy: Shared Disk - I have a discardable copy to read from disk
    Invalid-Remote: Not present on this machine in memory or disk
    Available?: Possibly never mapped before - is this a thing?

    Valid = ok to access memory, Invalid = access requires load (from disk or network)
      Valid/RO: Writeable/Dirty = page is newer than disk, write ok, RO/Clean = page is same as disk, fault on write
      Invalid/RO: Local = page is on disk, Remote = no record here
    Exclusive = local is only instance, Shared = other copies exist
    Master = local is permanent copy, Copy = another node has permanent copy

    Read-only/Read-Write?

VDEM = resident writeable unbacked (idle-> save:VCEM)
VCEM = resident writeable (write->unback:VDEM) backed
VDSM = resident read-only (write->exclusive:VDEM) unbacked (idle->save:VCSM)
VCSM = resident read-only (write->exclusive/unback:VDEM) backed
VDSC = resident read-only (write->exclusive:VDEM) unbacked (idle->save:VCSC)
VCSC = resident read-only (write->exclusive/unback:VDEM) backed
ILEM = paged out exclusive (read->pagein:VCEM; write->exclusive/unback:VDEM) del-locked
ILSM = paged out master (read->pagein:VCSM; write->exclusive/unback:VDEM) del-locked
ILSC = paged out copy (read->pagein:VCSC; write-> exclusive/unback:VDEM)
  0b10
IRSC = no record (read->fetch; write->fetch/exclusive:VDEM)
  encoding: 0
// VDEC = invalid (E->M)
// VCEC = invalid (E->M)
// IREM = invalid (R->S, R->C)
// IRSM = invalid (R->S, R->C)
// ILEC = invalid (E->M)
// IREC = invalid (R->S)


Broadcast: Reliable delivery, must respond


Redundant replication:
  track count of master copies
  hand off master pages to another node until threshold

Thread stats: (decaying over a period)
  reads:
  read-misses:
  writes:
  write-misses:

# VA Ranges (cluster wide)
ranges: distributed view of va ranges (also range security?)
  init(table_location)
  range: allocated and free virtual address ranges (cluster-wide)
    reserve(initial_pages): Result<pbase, err>
    extend(pbase, new_size): Result<pbase, err>
    release(pbase): Result<pbase, err)

# Node VA-PA translation (page table)
vm: maps va to pa
  init(base)
  idle()
  frame: used and free physical RAM (on the current node)
    allocate_fixed(pa, n)
    allocate(n)
    free(a, n)
  page: transalation tables for virtual addresses
    id_map(a, n, contig, pinned)
    map(a, n, contig, pinned)
    release(a, n)
    pin_remote(a, node)
  swap: migrate pages between memory and persistent storage
    in(a, frame)
    out(a, frame)    
  handlers: exception handlers
    data_fault()

# Node physical frames
frame table: track physical frames either in use or free
  bitmap of physical pages 4k page per 128Mb of mem
  init: 1 full table - all free
  allocate(n): n contig
    from bottom to top
      next if zero (all allocated)
      next if not n contiguous
      bit = bit with next n 1s
      page = page(word+bit)
      bits(page..+n) = 0 (allocated)
      return page
    out of mem
  free(a,n):
    page = page(a)
    word = word(page)
    bits(word)

# Decay

Bank of unsigned byte access histories, one per page frame
on page access fault:
  if access history == 0 then ensure not in warm pools (2nd chance)   
  OR MSB on page access
8 times each hot period length (eg. 8/s):
  decay all access history by byte.SHR
  clear all PTE.access bits and flush TSB
after each hot period, add all unaccessed or warm-% of LRU pages to warm pool
after each warm period (eg. 1/s):
  take remaining dirty pages to dirty pool
  take remaining clean pages to victims queue
  zero oldest victims and mark as free at zero-rate if under free target
  write oldest dirty pages and move to victims at write-rate if under victim target

examine distribution of access history bytes and tune hot period
tune (shorten and back-off) warm period to respond to change
tune warm-% for residual warm based on zero-rate change
dirty write-rate
victims zero-rate


Data structures:
Global:
- process control block (thread group; L0-L2 page table descriptors)
- thread control blocks (each thread; saved state; user stack; thread stats)
- virtual address space (address space section reservation/security; tree)

Local/banked (0xFFFF...):
- L3 page block descriptors (each instance; status (eg. VDEM); array)
- kernel stack (block)
- local page frame (free memory pages; address in each page; probOwner?; array)
- disk page frames (disk pages in use; probOwner?; array)
