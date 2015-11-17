.section .startup
.global _reset
.extern rust_main
_reset:
 LDR sp, =stack_top
 BL rust_main
 B .
