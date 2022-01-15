ENTRY(reset)
SECTIONS
{
    /DISCARD/ :
    {
        *(.ARM.exidx*)
    }

    . = 0xffffff8200080000;
    # . = 0x40080000;
    image_base = .;

    .text :
    {
        text_base = .;
        *(.startup)
        *(.handler)
        *(.text*)
        . = ALIGN(0x1000);
        text_end = .;
    }

    .rodata ALIGN(0x200000) :
    {
        static_base = .;
        *(.rodata*)
        . = ALIGN(0x1000);
        static_end = .;
    }

    .data ALIGN(0x200000) :
    {
        data_base = .;
        *(.data*)
    }

    .bss ALIGN(0x1000) :
    {
        bss_base = .;
        *(.bss*)
        . = ALIGN(0x1000);
        bss_end = .;
    }

    data_end = .;

    .stack ALIGN(0x200000) :
    {
        stack_base = .;
        . = . + 0x8000;
        stack_end = .;
    }
    image_end = .;
}
