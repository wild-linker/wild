MEMORY {
    code : ORIGIN = 0x10000, LENGTH = 0x10000
    data : ORIGIN = 0x400000, LENGTH = 0x10000
}

SECTIONS {
    .text : { *(.text*) } > code
    .data : { KEEP(*(.data.ram1)) } > data
    /DISCARD/ : { *(.note.*) }
}
