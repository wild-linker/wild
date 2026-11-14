ENTRY(_start)

SECTIONS {
  .text 0x10000 : { *(.text.start) *(.text.caller) }
  .far 0x8010018 : { *(.far) }
}
