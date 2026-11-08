ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    . = 0x500000;
    .custom_data : { *(.custom_data) }
    . = 0x600000;
    .data : { *(.data .data.*) }
    . = 0x700000;
    .bss : { *(.bss .bss.*) }
}

ASSERT(ADDR(.init_array) < ADDR(.custom_data) || ADDR(.init_array) >= ADDR(.data), ".init_array placed inside .custom_data segment")
