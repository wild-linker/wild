ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .first : {
        first_start = .;
        *(.first*)
        first_end = .;
    }
    . = 0x500000;
    .text : {
        text_start = .;
        *(.text*)
        text_end = .;
    }
    . = 0x600000;
    .middle : {
        middle_start = .;
        *(.middle*)
        middle_end = .;
    }
    . = 0x700000;
    .data : {
        data_start = .;
        *(.data*)
        data_end = .;
    }
    . = 0x800000;
    .last : {
        last_start = .;
        *(.last*)
        last_end = .;
    }
    .bss : { *(.bss*) }
}
