ENTRY(_reset)
SECTIONS
{

 /DISCARD/ : 
 {
     *(.ARM.exidx*)
 }
 
 . = 0x10000;
 .text : 
 { 
     *(.startup) 
     *(.text*) 
 }
 
 . = ALIGN(0x1000);
 stack = .;
 . = . + 0x1000; /* 4kB of stack memory */
 stack_top = .;
}
