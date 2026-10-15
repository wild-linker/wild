ENTRY(_start)

PHDRS {
    text PT_LOAD FILEHDR PHDRS;
    data PT_LOAD;
}

SECTIONS {
    . = SEGMENT_START("text-segment", 0x400000) + SIZEOF_HEADERS;
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

ASSERT(ADDR(.text) == 0x400000 + SIZEOF_HEADERS, "text should start at 0x400000 + SIZEOF_HEADERS");
