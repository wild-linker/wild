ENTRY(_start)

SECTIONS {
    . = 0x100000;
    .text : { *(.text .text.*) }
    .foo 0x200000 : AT(0x500000) { *(.foo .foo.*) }
    .bar : { *(.bar .bar.*) }
}

ASSERT(LOADADDR(.foo) == 0x500000, "LMA of .foo should be 0x500000");
ASSERT(LOADADDR(.bar) == LOADADDR(.foo) + SIZEOF(.foo), "LMA of .bar should continue after .foo");
