SECTIONS {
    . = 0x400000;
    .text : { *(.text .text.*) }
    .foo : { *(.foo.other) }
}

ASSERT(ADDR(.foo) == 0x600000, ".foo address override was not applied")
