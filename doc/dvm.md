
# thesis
1.  One shared view of available address ranges.

2.  Local per-node view of page translation
    A VA will map to different pages on different nodes.
    No concept of process addressing (all pointers work)

3.  Per node view of frames:
    The organisation of its physical memory is managed by the node.

A set of translation tables per

# state of local ownership of a page
    (The state is recorded in our )
    Exclusive: I am the only machine with a copy
    Shared: I am one of many machines with a copy
    Remote: Exclusive or Shared, but elsewhere
    Available?: Possibly never mapped before - is this a thing?

Broadcast: Reliable delivery, must respond



Redundant replication:

Thread stats: (decaying over a period)
  reads:
  read-misses:
  writes:
  write-misses:

# VA Ranges (cluster wide)
ranges: distributed view of va ranges (also range security?)
  init(base)
  range: allocated and free virtual address ranges (cluster-wide)
    reserve(initial_size)
    extend(base, new_size)
    release(base)

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
