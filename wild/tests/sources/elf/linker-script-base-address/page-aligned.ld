ENTRY(_start)

PHDRS {
    text PT_LOAD FILEHDR PHDRS;
    data PT_LOAD;
}

SECTIONS {
    . = 0x600000;
    .text : {
        *(.text*)
    } :text
    . = 0x800000;
    .data : {
        *(.data*)
    } :data
    .bss : {
        *(.bss*)
    } :data
}

ASSERT(ADDR(.text) == 0x600000, "text should start at or after base address");
