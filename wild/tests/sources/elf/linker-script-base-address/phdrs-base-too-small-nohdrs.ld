ENTRY(_start)

PHDRS {
    text PT_LOAD;
}

SECTIONS {
    . = 0x10;
    .text : {
        *(.text*)
    } :text
}
