SECTIONS {
    .text : ALIGN(1 + (0 && (1 / 0))) {
        *(.text*)
    }
}
