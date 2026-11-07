ENTRY(begin_here)

SECTIONS
{
    . = 0x600000;
    start_of_sections = .;
    .data 0x800000 : {
        start_of_data = .;
        *(.data)
        . = ALIGN(512);
        start_of_512 = .;
        *(.data.2)
    }
    .bss : { *(.bss) }
}

ASSERT(ADDR(.text) > 0x600000, "text not at expected address");
ASSERT(ADDR(.text) < 0x800000, "text not at expected address");
