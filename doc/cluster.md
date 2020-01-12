### Booting into the cluster
  - networking bootloader
  - scan for clusters
  - ask random cluster member for size estimates
  - choose most populous cluster
  - send unique id to ask for:
    - a node id (vm base for my node space)  
    - boot thread id (vm base for tcb) and
    - vm control (vm base for address ranges)
  - init vm into my node space (page and frame tables)
  - map boot thread
  - map
  - set up tcb for boot thread

  * node on

  - create an idle thread


# initial boot

first thread is boot2 (brom, uboot)

## modules
  pager - memory manager
  thread - task switcher
  handler - interrupts
  device - io manager

  user1 - first new process



## vm, scheduler
