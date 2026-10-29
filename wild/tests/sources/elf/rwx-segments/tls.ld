PHDRS {
    text PT_LOAD FLAGS(5);
    data PT_LOAD FLAGS(6);
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
