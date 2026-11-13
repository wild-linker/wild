PHDRS {
    text PT_LOAD FILEHDR PHDRS;
    data PT_LOAD;
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
