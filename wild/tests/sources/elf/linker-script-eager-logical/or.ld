SECTIONS {
    .text : ALIGN(1 || (1 / 0)) {
        *(.text*)
    }
}
