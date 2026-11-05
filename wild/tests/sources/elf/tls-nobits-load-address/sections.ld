SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    .tbss : { *(.tbss .tbss.*) }
    .data : { *(.data .data.*) }
}

ASSERT(LOADADDR(.data) == ADDR(.data), ".data LMA differs from VMA after .tbss")
