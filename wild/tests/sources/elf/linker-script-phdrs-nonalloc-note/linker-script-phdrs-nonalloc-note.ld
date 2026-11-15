ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(5);
    data PT_LOAD FLAGS(6);
    note PT_NOTE;
    tls PT_TLS;
    null_seg PT_NULL;
}

SECTIONS {
    . = 0x400000;
    .text : { *(.text) } :text
    .note.one : { KEEP(*(.note.one)) } :text :note
    .debug_test : { KEEP(*(.debug_test)) }
    .note.two : { KEEP(*(.note.two)) }
    .null.one : { KEEP(*(.null.one)) } :text :null_seg
    .debug_test4 : { KEEP(*(.debug_test4)) }
    .null.two : { KEEP(*(.null.two)) }

    . = 0x500000;
    .data.one : { KEEP(*(.data.one)) } :data
    .debug_test2 : { KEEP(*(.debug_test2)) }
    .data.two : { KEEP(*(.data.two)) }
    .tdata.one : { KEEP(*(.tdata.one)) } :data :tls
    .debug_test3 : { KEEP(*(.debug_test3)) }
    .tdata.two : { KEEP(*(.tdata.two)) }

    /DISCARD/ : { *(.comment) *(.eh_frame) *(.note.gnu.property) }
}
