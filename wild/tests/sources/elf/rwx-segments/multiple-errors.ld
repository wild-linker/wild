PHDRS {
    text PT_LOAD FLAGS(7);
    data PT_LOAD FLAGS(7);
    tls PT_TLS FLAGS(1);
}

SECTIONS {
    . = 0x400000;
    .text : { *(.text) } :text
    . = ALIGN(0x1000);
    .data : { *(.data) } :data
    .tdata : { *(.tdata) } :data :tls
    .bss : { *(.bss) } :data
}
