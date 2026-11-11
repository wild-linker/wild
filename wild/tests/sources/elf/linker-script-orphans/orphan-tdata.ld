SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    .tdata : { *(.tdata) }
    .tbss : { *(.tbss .tbss.*) }
    .data : { *(.data .data.*) }
}
