.section        .startup
.global         _reset
.extern         rust_main

_reset:         mrs     x7, CurrentEL
                mrs     x6, CPACR_EL1
                orr     x6, x6, 0x100000
                msr     CPACR_EL1, x6

	        ldr     x11, =stack_top
	        mov     sp, x11

                ldr     x10, =rust_main
                br      x10
                b       .
