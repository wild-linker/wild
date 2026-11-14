ENTRY(_start)

SECTIONS {
  .text 0x10000 : { *(.text.low) }
  .high 0x8010008 : { *(.text.high) }
}
