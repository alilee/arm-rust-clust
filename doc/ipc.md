

send: async send package to thread, interrupt if running, but not if masked

thread:
running:  inter-processor interrupt, software generated interrupt, GIC, re-entrant?
ready:    tweak tcb/stack so wakes up in handler 
sleeping: tweak tcb/stack so wakes up in handler, returns to sleep (unless overridden)

boost priority?
