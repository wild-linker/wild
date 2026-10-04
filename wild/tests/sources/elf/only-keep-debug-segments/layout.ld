PHDRS {
    load PT_LOAD FILEHDR PHDRS FLAGS(5);
    notes PT_NOTE FLAGS(4);
    tls PT_TLS FLAGS(4);
    empty PT_LOAD FLAGS(6);
}
SECTIONS {
    /DISCARD/ : { *(.note.gnu.property) }
    .note.first 0x500000 : { *(.note.first) } :load :notes
    .note.second : { *(.note.second) } :load :notes
    .text : { *(.text) } :load
    .tdata : { *(.tdata) } :load :tls
    .tbss : { *(.tbss) } :load :tls
    .data : { *(.data) } :empty
}
