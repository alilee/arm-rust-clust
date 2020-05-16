ENTRY(_reset)
SECTIONS
{

    /DISCARD/ :
    {
        *(.ARM.exidx*)
    }

    . = 0x40080000;
    image_base = .;

    .text :
    {
        *(.startup)
        *(.handler)
        *(.text*)
    }

    .rodata :
    {
        *(.rodata*)
    }

    .data :
    ALIGN(0x1000)
    {
        stack = .;
        . = . + 0x1000;
        STACK_TOP = .;
        . = . + 0x1000;
        data_gap = .;
        . = . + 0x1000;    /* separates data and linker globals for debugging */
    }

    .bss :
    {
        *(.bss*)
    }

    image_end = ALIGN(4096);
}
