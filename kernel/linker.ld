ENTRY(_start)

SECTIONS
{
    /* Use a high virtual address for the kernel (standard for 64-bit kernels) */
    . = 0xffffffff80000000;

    .text :
    {
        /* Ensure Limine requests are kept */
        KEEP(*(.limine_reqs))
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        *(.bss .bss.*)
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}
