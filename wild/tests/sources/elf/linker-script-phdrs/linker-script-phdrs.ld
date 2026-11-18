PHDRS {
    text PT_LOAD FILEHDR PHDRS;
    dynamic PT_DYNAMIC;
    empty PT_LOAD;
    data PT_LOAD FLAGS(6);   /* R + W */
    rodata PT_LOAD FLAGS(4); /* R */
}

SECTIONS {
    . = SIZEOF_HEADERS;
    start_of_sections = .;
    .text : {
        *(.text*)
    } :text

    .data : {
        *(.data*)
    } :data

    .bss : {
        *(.bss*)
    }

    .custom : {
        *(.custom)
    } :NONE

    .dynamic : {
        *(.dynamic)
    } :rodata :dynamic

    .rodata : {
        *(.rodata*)
    } :rodata :NONE
}

header_size = is_64_bit ? 0x40 : 0x34;
pheader_size = is_64_bit ? 0x38 : 0x20;

ASSERT(start_of_sections == header_size + ((5 + is_riscv) * pheader_size), "Wrong start of sections");
ASSERT(SIZEOF_HEADERS == header_size + ((5 + is_riscv) * pheader_size), "Wrong size of headers");
