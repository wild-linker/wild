SECTIONS {
    . = 0x400000;
    .text : { *(.text) }
    .data : { *(.data .data.*) }
    .text : { *(.text.second) }
}
