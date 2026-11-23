MEMORY {
    ROM : ORIGIN = 0x100000, LENGTH = 64K
    RAM : ORIGIN = 0x200000, LENGTH = 64K
}

SECTIONS {
    .text : { KEEP(*(.text .text.*)) } > ROM
    .first : { KEEP(*(.first)) } > RAM
    .bss : { KEEP(*(.bss .bss.*)) } > RAM
}
