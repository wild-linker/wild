ENTRY(_start)

SECTIONS {
    .text 0x600000 : {
        *(.text)
    }
    . = 0x700000;
    .other : {
        KEEP(*(.other))
    }
    . = 0x500000;
    .rodata : {
        KEEP(*(.rodata.*))
    }
}
