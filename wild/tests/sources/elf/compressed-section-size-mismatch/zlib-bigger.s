// The compression header says 4096 bytes, but the stream decompresses to 16. The section isn't
// named .debug_info because assemblers won't set SHF_COMPRESSED (0x800) on known section names.
.section .debug_test,"0x800",%progbits
// ch_type, ch_reserved, ch_size, ch_addralign
  .long 1
  .long 0
  .quad 4096
  .quad 1
  .byte 0x78,0x9c,0x63,0x60,0x40,0x5,0x0,0x0,0x10,0x0,0x1
.section .note.GNU-stack,"",%progbits
