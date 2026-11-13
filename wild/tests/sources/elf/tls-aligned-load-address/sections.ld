SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    .data : { *(.data .data.*) }
    .tdata : { *(.tdata .tdata.*) }
    .tbss : { *(.tbss .tbss.*) }
}

ASSERT(LOADADDR(.tdata) == ADDR(.tdata), ".tdata LMA differs from VMA after TLS alignment")
