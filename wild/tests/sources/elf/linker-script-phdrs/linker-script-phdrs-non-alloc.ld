PHDRS {
    text PT_LOAD FLAGS(5) FILEHDR PHDRS;
    data PT_LOAD FLAGS(6);
}

SECTIONS {
    . = SIZEOF_HEADERS;
    .text : {
        *(.text*)
    } :text

    .data : {
        *(.data*)
    } :data

    .comment : {
        *(.comment)
    }
}
