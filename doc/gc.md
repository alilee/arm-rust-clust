
gc'd memory-range
 - heaps:
   for each page, which heap size is it in
 - mark:
   block sweeper
   clear all
   traverse
   odd? no
   out of range?
   mark
   resume sweeper
 - sweep:
   accessed? yes
   add to free list for size
 - allocator: 
   find free list of right size
   if empty?
   extend allocate page.

lsb 1 = integer
lsb 0 = objectref (pointer to object structure)

object:
aligned 4-byte:
word 0: [type]:[actual size in bytes]
word 1: full words of data... 

types:
  0: free
  1: fixed  
  2: string
  10: cons
  
eg. 
  [0]:[4],next free
  [1]:[5], 0x12121212, 0x12000000, 0x00000000
  [10]:[8], objref, next
  
  
