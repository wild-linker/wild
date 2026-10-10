ENTRY(begin_here)

first = second;
second = first;

SECTIONS
{
    . = first + SIZEOF_HEADERS;
    .text : {
        *(.text)
    }
    .data : {
        *(.data)
    }
    .bss : {
        KEEP(*(.bss))
    }
}
