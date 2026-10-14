ENTRY(_start)
SECTIONS {
    .text 0x400ff8 : { KEEP(*(.text)) }
    .data (_start + 0x10000) : { KEEP(*(.data)) }
}
ASSERT(ADDR(.data) == _start + 0x10000, "wrong early symbol address")
