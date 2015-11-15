ENTRY(_reset)
SECTIONS
{
 . = 0x10000;
 .startup . : { startup.o(.text) }
 .text : { *(.text) }
 .data : { *(.data) }
 .bss : { *(.bss COMMON) }
 . = ALIGN(0x1000);
 stack = .;
 . = . + 0x1000; /* 4kB of stack memory */
 stack_top = .;
 . = ALIGN(0x4000);
 trans_table = .;
 . = . + 0x6800; /* 18kB, 16kb L1 and 2 1k L2 pages */
 trans_table_top = .;
 . = ALIGN(0x1000);
 page_map = .;
 . = . + 0x1000; /* 4kB of 1 bit per page */
 page_map_top = .;
}
