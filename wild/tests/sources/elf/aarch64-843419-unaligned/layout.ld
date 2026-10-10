ENTRY(_start)
SECTIONS {
    .text 0x400ff8 : { KEEP(*(.text)) }
    .data 0x410000 : { KEEP(*(.data)) }
}
