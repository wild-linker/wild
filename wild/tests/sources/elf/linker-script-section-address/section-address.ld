SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    . = 0x600000;
    .data 0x800000 : { *(.data .data.*) }
}
