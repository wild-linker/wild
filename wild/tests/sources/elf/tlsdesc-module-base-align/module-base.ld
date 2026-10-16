ENTRY(main)

SECTIONS {
    /* Exercise early symbol evaluation without depending on the provisional TLS address. */
    . = 0x10000 + (_TLS_MODULE_BASE_ & 7);
    script_start = .;
    .text : { *(.text .text.*) }
    .tdata : { *(.tdata .tdata.*) }
}
