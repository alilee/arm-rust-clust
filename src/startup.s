.section .startup
.global _reset
.extern rust_main
_reset:
	ldr x11, =stack_top
	mov sp, x11

    ldr x10, =rust_main
    br x10
    b .
