# Page replacement

User pages:

Kernel queues:
- Zero // the zero page
- Zeroed // ready to allocate
- Zeroing // getting cleansed (Free to Zeroed)
- Free // no longer required/resident
- Kernel // pinned for kernel

Two user queues:
- Warm
- Cold

## Events

### Access fault (in L3, because we do not !AF non-leaf blocks)

1. Set AF in L3 PTE (so that Accessed fault isn't triggered until the AF is cleared).
2. Move accessed page from its current queue to the top of Warm.
3. Increment warm count

### Permission fault at L3 or access fault, and writing (ESR_EL1::WnR == 1)
5. Set Dirty in L3 PTE (so will be saved when paged out)
6. Set AP::ReadWrite
7. Invalidate TLB entry

### And then, if Warm count is greater than target:
3. Move whole Warm queue to the front of Cold.
4. Zero warm count
5. Clear AF in L3 PTE for each Warm page. Ensure AF faults don't break traversal of the Cold list by reseting "behind" the traversal.
6. Clear and set a low-priority IO queue to save/clean the dirty pages in the tail of Cold. 

### Settings

- Zero: target length of pages - 6
- CleanCold: target length of pages - 6
- Cold: % of user (Hot/Warm/Cold) pages - 30%
- Warm: Warm/Hot - 100%
- Max time-slice length change (%/ts): 10%
- Max/reset/min time-slice length: 1000ms, 200ms, 100ms
- Depth of Cold to low-priority pre-clean