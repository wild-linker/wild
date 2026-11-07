ENTRY(_start)
PROVIDE(__global_pointer$ = 0);
SECTIONS {
    .text 0x500000 : { *(.text .text.*) }
    .note.gnu.property 0x570000 : { *(.note.gnu.property) }
    .eh_frame 0x580000 : { *(.eh_frame) }
    .got 0x600000 : { *(.got) }
    .debug_line 0 : { KEEP(*(.debug_line)) }
}
