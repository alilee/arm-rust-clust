
state:
  init:     state not consistent yet
  running:  scheduled onto a core
  ready:    waiting for scheduler to assign a core
  waiting:  wait for an event to complete (ipc - io)
  sleeping: waiting until a future time 
  complete: halted complete, clean-up
  error:    halted error state


each thread is known by the virtual address of its tcb. 


tcb contents:
  running:   priority
  ready:     saved state
  sleeping:  saved state

priority:
  base priority
  override priority (ie. if higher priority thread is waiting on it)
  dynamic boost? (ie. return from long sleep)

saved state:
  registers
  signal queue
  accounting
  
accounting:
  cpu-usage:
    cycles
    running-time
    periodic: current-, last-, long- term
        cpu-time
        ready-time
        waiting-time
        sleeping-time
        vm-time-attribution
  working set:
    rolling historic periods:
      read-faults-persist
      read-faults-remote x host
      write-faults-persist
      write-faults-remote x host



scheduler:

priority bands:

  rt:           all run until none ready, will starve user
  user:         prioritised expontential, will starve idle
  idle:         spare capacity after higher threads waiting or sleeping, will starve rest
  rest:         last resort - save power      

global accounting:
  periodic:
      kernel-time           time in handlers?
      vm-time               time in vm-handlers
      rt-time               time in threads at rt priorities
      user-time             time in user threads
      idle-time             time in idle threads
      rest-time             time resting
      faults
      read-faults-local     no.
      read-faults-remote    no. 
  current:
    physical mem free       pages available
    local storage free      pages available
  