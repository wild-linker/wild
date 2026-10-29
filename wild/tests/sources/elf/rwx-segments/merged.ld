SECTIONS {
    . = 0x400000;
    .merged : { *(.text) *(.data) }
}
