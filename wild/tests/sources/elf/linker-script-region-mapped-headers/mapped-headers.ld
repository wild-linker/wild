MEMORY {
    code : ORIGIN = 0x20000, LENGTH = 0x10000
    data : ORIGIN = 0x400000, LENGTH = 0x10000
}

PHDRS {
    text PT_LOAD FILEHDR PHDRS FLAGS(5);
    data PT_LOAD FLAGS(6);
}

SECTIONS {
    .text ORIGIN(code) + 0x1000 : {
        *(.text*)
        . = ALIGN(0x1000);
    } > code :text
    .data : { KEEP(*(.data.ram1)) } > data :data
    /DISCARD/ : { *(.note.*) }
}
