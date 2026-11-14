ENTRY(_start)

SECTIONS {
  .text 0x10000 : { *(.text) }
  .far 0x8010004 : { *(.far) }
}
