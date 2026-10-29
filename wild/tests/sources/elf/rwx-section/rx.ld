PHDRS {
    text PT_LOAD FLAGS(5);
}

SECTIONS {
    . = 0x400000;
    .wtext : { *(.wtext) } :text
}
