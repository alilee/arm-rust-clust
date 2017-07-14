.section        .startup
.global         _reset
.extern         boot2, stack_top

_reset:         mrs     x7, CurrentEL
                mrs     x6, CPACR_EL1
                orr     x6, x6, 0x100000
                msr     CPACR_EL1, x6

	        ldr     x11, stack_top
	        mov     sp, x11

                ldr     x10, boot2
                br      x10

                b       .
