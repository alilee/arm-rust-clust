
/*

threads: boot, repl, idle


1. we have a stack

2. start mmu
  - map translation table
  - map empty page map
  - map boot_text, existing-identity
  - map idle_text, existing
  - switch on vmsa

3. exception vector table
  - map kernel stack and enable
  - map table, handlers at hivecs
  - hivecs SCTLR.V = 1
  - exception handlers:
    - accessed
    - page fault
    - tick
    - service

4. tcb, stack and scheduler
   - map boot tcb
   - map boot stack
   - reserve scheduler
   - seed boot as running

5. initialise timer

6. spawn idle thread

7. spawn repl thread

8. terminate boot

*/

#include "vm.h"

extern uint32_t *reset;
extern uint32_t reset_length;

void boot() {
    
  vm_seed_trans_table();
  vm_seed_page_map();

  vm_seed((uint32_t*)reset, (uint32_t*)reset, reset_length);
      
  while (1) {
    
  }
}
