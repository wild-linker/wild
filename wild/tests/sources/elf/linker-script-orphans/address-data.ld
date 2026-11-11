SECTIONS {
    . = 0x400000;
    .data 0x800000 : { *(.data .data.*) }
}
