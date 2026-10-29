PHDRS {
    text PT_LOAD FLAGS(5);
    data PT_LOAD FLAGS(6);
    bss PT_LOAD FLAGS(7);
}

SECTIONS {
    . = 0x400000;
    .text : { *(.text) } :text
    . = ALIGN(0x1000);
    .data : { *(.data) } :data
    . = ALIGN(0x1000);
    .bss : { *(.bss) } :bss
}
