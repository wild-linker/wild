PHDRS {
    text PT_LOAD FLAGS(5);
    data PT_LOAD FLAGS(6);
    empty PT_LOAD FLAGS(7);
    tls PT_TLS FLAGS(1);
}

SECTIONS {
    . = 0x400000;
    .text : { *(.text) } :text
    . = ALIGN(0x1000);
    .data : { *(.data) } :data
    .bss : { *(.bss) }
    .empty : { *(.empty) } :empty
    /DISCARD/ : { *(.note.gnu.property) }
}
