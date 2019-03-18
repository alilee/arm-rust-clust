

send: async send package to thread, interrupt if running, but not if masked

thread:
running:  inter-processor interrupt, software generated interrupt, GIC, re-entrant?
ready:    tweak tcb/stack so wakes up in handler
sleeping: tweak tcb/stack so wakes up in handler, returns to sleep (unless overridden)

boost priority?


/// push page with signal
/// send signal to thread, ensuring page is accessible according to permissions.
/// message is ack'd
send(thread_id, signal, page, access)::
  thread_id: recipient TCB
  signal: u32
  location: address (drags entire 4Kb page, unless 0)
  permissions: new page ownership (ie. push r/w: VDEM (leaves IRSC), push r/o: VDSC (leaving VxSC))
