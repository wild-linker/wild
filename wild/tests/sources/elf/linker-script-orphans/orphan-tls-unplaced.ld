ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    . = 0x500000;
    .data : { *(.data .data.*) }
    . = 0x600000;
    .bss : { *(.bss .bss.*) }
}

ASSERT(ADDR(.tdata) < ADDR(.bss), ".tdata placed after .bss")
