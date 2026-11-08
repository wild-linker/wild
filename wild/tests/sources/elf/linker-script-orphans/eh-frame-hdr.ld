ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    . = 0x500000;
    .eh_frame : { *(.eh_frame) }
    . = 0x600000;
    .data : { *(.data .data.*) }
    . = 0x700000;
    .bss : { *(.bss .bss.*) }
}

ASSERT(ADDR(.eh_frame_hdr) < ADDR(.data), ".eh_frame_hdr placed after .data")
