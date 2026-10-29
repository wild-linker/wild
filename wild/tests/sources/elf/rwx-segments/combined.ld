PHDRS {
    combined PT_LOAD;
}

SECTIONS {
    . = 0x400000;
    .text : { *(.text) } :combined
    .data : { *(.data) }
    .bss : { *(.bss) }
}
