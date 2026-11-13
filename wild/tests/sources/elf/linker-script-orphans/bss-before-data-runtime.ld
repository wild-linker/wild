ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .text : {
        *(.text .text.*)
        /* Keep writable sections off executable pages on all test architectures. */
        . = ALIGN(0x10000);
    }
    .bss : { *(.bss .bss.* .sbss .sbss.*) }
    .data : {
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.data .data.* .sdata .sdata.*)
    }
}
