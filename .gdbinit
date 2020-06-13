display/i $pc

# layout asm
# layout regs
set step-mode on

break _reset
break el1_sp1_sync_handler
break kernel_main
