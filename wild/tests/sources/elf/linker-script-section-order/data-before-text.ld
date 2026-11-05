ENTRY(_start)

SECTIONS {
    . = 0x500000;
    .data : {
        data_start = .;
        *(.data*)
        data_end = .;
    }
    . = 0x502000;
    .got : {
        *(.got)
    }
    .toc : {
        *(.toc*)
    }
    . = 0x600000;
    .text : {
        text_start = .;
        *(.text*)
        text_end = .;
    }
    . = 0x700000;
    .rodata : {
        rodata_start = .;
        *(.rodata*)
        rodata_end = .;
    }
    . = 0x800000;
    .bss : {
        bss_start = .;
        *(.bss*)
        bss_end = .;
    }
}
