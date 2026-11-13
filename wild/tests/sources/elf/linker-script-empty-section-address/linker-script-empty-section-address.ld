ENTRY(_start)

SECTIONS {
    . = 0x400000;
    .text : {
        *(.text)
    }

    .empty 0x500000 : {
        mark = .;
    }

}
