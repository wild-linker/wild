ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    . = 0x500000;
    .custom_data : { *(.custom_data) }
    .data : { *(.data .data.*) }
    .bss : { *(.bss .bss.*) }
}

ASSERT(ADDR(.data) - ADDR(.custom_data) == 4, ".data not contiguous with .custom_data")
