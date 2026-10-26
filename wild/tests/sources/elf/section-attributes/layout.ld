ENTRY(_start)
SECTIONS {
  . = 0x400000;
  .text : { *(.text) }
  .rodata : { *(.rodata.plain) *(.rodata.str*) *(.rodata.cst*) }
}
