.global _reset
_reset:
 LDR sp, =stack_top
 BL boot
 B .
