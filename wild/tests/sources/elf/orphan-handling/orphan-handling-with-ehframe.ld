ENTRY(_start)

SECTIONS {
    . = 0x600000;
	.eh_frame : {
		*(.eh_frame)
	}
    .text : {
        *(.text*)
    }
    . = 0x800000;
    .data : {
        *(.data*)
    }
    .bss : {
        *(.bss*)
    }
}
