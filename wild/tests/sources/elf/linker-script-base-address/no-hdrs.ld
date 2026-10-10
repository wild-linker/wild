ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(7);
}

SECTIONS {
    . = 0x10000000;
    .text : {
        KEEP(*(.text.first))
        . = symbol1 | (1 << 3) | (1 << 8) | (1 << 14);
        KEEP(*(.text.second))
        *(.text*)
    } :text

    /DISCARD/ : {
        *(.note.gnu.property)
    }
}

ASSERT((((symbol1) | symbol2) -
  (symbol1 & symbol2)) == ((1 << 3) | (1 << 8) | (1 << 14)),
  "Symbol pair address calculation failed");
