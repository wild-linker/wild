MEMORY {
    ROM : ORIGIN = 0x100000, LENGTH = 64K
    RAM : ORIGIN = 0x200000, LENGTH = 64K
}

SECTIONS {
    .text : { *(.text .text.*) } > ROM
    .data : { *(.data .data.*) } > RAM
    .bss : { *(.bss .bss.*) } > RAM
}
