ENTRY(_start)

MEMORY {
    ROM : ORIGIN = 0x500000, LENGTH = 0x100000
    RAM : ORIGIN = 0x800000, LENGTH = 0x100000
}

SECTIONS {
    .text 0x510000 : {
        *(.text .text.*)
        . = ALIGN(0x10000);
    } > ROM
    .bss : { *(.bss .bss.* .sbss .sbss.*) } > RAM
    .data : {
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.data .data.* .sdata .sdata.*)
    } > RAM
    /DISCARD/ : { *(.note.* .riscv.attributes) }
}
