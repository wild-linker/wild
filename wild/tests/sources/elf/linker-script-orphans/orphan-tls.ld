SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    .tdata : { *(.tdata .tdata.*) }
    .data : { *(.data .data.*) }
}
