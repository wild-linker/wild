ENTRY(_start)

SECTIONS {
    . = 0x401000;
    .text : { *(.text*) }
    . = ALIGN(0x1000);
    .rodata : {
        KEEP(*(.rodata.kept))
        *(.rodata*)
    }
    . = ALIGN(0x1000);
    .data : { *(.data*) }
    .bss : { *(.bss*) }
}
