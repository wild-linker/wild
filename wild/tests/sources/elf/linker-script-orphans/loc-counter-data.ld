SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) . = ALIGN(4096); }
    . = 0x500000;
    .bss : { *(.bss .bss.*) }
}

ASSERT(ADDR(.bss) == 0x500000, "orphan data consumed the .bss location counter")
