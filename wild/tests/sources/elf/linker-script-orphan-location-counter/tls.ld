SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    . = 0x500000;
    .tbss : { *(.tbss .tbss.*) }
    .data : { *(.data .data.*) }
}

ASSERT(ADDR(.tbss) == 0x500000, "orphan .tdata consumed the .tbss location counter")
