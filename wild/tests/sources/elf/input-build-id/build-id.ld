ENTRY(_start)

SECTIONS {
  . = 0x400000;
  .note.gnu.build-id : { *(.note.gnu.build-id) }
  .text : { *(.text .text.*) }
  .rodata : { *(.rodata .rodata.*) }
  .data : { *(.data .data.*) }
  .bss : { *(.bss .bss.*) }
}
