ENTRY(_start)

SECTIONS {
    . = 0x600000;
    .text : {
        *(.text*)
    }
    . = 0x800000;
    .data : {
        *(.data*)
    }
    .bss : {
        *(.bss*)
    }
}
