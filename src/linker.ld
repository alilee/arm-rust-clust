ENTRY(_reset)
SECTIONS
{
 . = 0x10000;
 .startup . : { startup.o(.text) }
 .text : { *(.text) *(.text.boot) }
 .data : { *(.data) }
 .bss : { *(.bss COMMON) }
 . = ALIGN(0x1000);
 stack = .;
 . = . + 0x1000; /* 4kB of stack memory */
 stack_top = .;
}
