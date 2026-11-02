SECTIONS {
    . = 0x400000;
    .text : { *(.text) text1_end = .; }
    .data : { *(.data .data.*) }
    .text : { text2_start = .; *(.text.second) }
}

ASSERT(text2_start - text1_end >= 4, "text2_start - text1_end is too small")
