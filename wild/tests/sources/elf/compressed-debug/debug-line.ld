ENTRY(_start)
PROVIDE(__global_pointer$ = 0);
SECTIONS {
    .text 0x500000 : { *(.text .text.*) }
    .eh_frame : { *(.eh_frame) }
    .got 0x600000 : { *(.got) }
    .debug_line 0 : { KEEP(*(.debug_line)) }
}
