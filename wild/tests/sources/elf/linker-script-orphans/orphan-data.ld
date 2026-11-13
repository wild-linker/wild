ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    . = 0x500000;
    .data : { *(.data .data.*) }
    .bss : { *(.bss .bss.*) }
}
