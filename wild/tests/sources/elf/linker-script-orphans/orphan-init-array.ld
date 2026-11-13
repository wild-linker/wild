ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    . = 0x500000;
    .data : { *(.data .data.*) }
    . = 0x600000;
    .bss : { *(.bss .bss.*) }
}

ASSERT(ADDR(.init_array) < ADDR(.bss), ".init_array placed after .bss")
