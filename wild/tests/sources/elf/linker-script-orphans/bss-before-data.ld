ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    .bss : { *(.bss .bss.*) }
    .data : { *(.data .data.*) }
}

ASSERT(ADDR(.init_array) < ADDR(.bss), ".init_array placed after .bss")
