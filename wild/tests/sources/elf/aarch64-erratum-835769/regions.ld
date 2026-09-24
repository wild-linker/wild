/* Data is just large enough for content */
MEMORY {
    TEXT : ORIGIN = 0x200000, LENGTH = 0x10000
    DATA : ORIGIN = 0x300000, LENGTH = 8
}

SECTIONS {
    .text : { *(.text) } > TEXT
    .data : { *(.data) } > DATA
}
