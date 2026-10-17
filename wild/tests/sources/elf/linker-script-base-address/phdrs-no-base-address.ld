ENTRY(_start)

PHDRS {
    text PT_LOAD FILEHDR PHDRS;
    data PT_LOAD;
}

SECTIONS {
    .text 0x400000 : {
        *(.text*)
    } :text
    .data 0x800000 : {
        *(.data*)
    } :data
    .bss : {
        *(.bss*)
    } :data
}

ASSERT(ADDR(.text) == 0x400000, "text should start at its explicit address");
