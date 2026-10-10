ENTRY(_start)

SECTIONS {
    . = 0x10;
    .text : {
        *(.text*)
    }
}
