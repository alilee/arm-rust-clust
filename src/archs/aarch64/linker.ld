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
        text_base = .;
        *(.startup)
        *(.handler)
        *(.text*)
    }

    .rodata ALIGN(0x1000) :
    {
        text_end = .;
        static_base = .;
        *(.rodata*)
    }

    .data ALIGN(0x1000) :
    {
        static_end = .;
        data_base = .;
        stack = ALIGN(0x1000);
        . = . + 0x1000;
        STACK_TOP = .;
        . = . + 0x1000;
        data_gap = .;
        . = . + 0x1000;    /* separates data and linker globals for debugging */
        *(.data*)
    }

    .bss ALIGN(0x1000) :
    {
        *(.bss*)
    }

    image_end = ALIGN(0x1000);
    data_end = .;
}
