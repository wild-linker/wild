ENTRY(_start)

PHDRS {
    text PT_LOAD FILEHDR PHDRS;
}

SECTIONS {
    . = 0x10;
    .text : {
        *(.text*)
    } :text
}
