ENTRY(_reset)
SECTIONS
{

    /DISCARD/ :
    {
        *(.ARM.exidx*)
    }

    . = 0xffffff8200080000;

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
        *(.data*)
    }

    .bss ALIGN(0x1000) :
    {
        *(.bss*)
    }

    image_end = ALIGN(0x1000);
    data_end = .;
}
