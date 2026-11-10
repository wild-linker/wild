ENTRY(_start)

MEMORY {
    ROM : ORIGIN = 0x500000, LENGTH = 0x100000
    RAM : ORIGIN = 0x800000, LENGTH = 0x100000
}

SECTIONS {
    .text : {
        *(.text .text.*)
        . = ALIGN(0x10000);
    } > ROM
    .data1 : {
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.data1 .data1.*)
    } > RAM
    .data2 0x810000 : {
        *(.data2 .data2.*)
    } > RAM
    /DISCARD/ : { *(.note.* .riscv.attributes) }
}
