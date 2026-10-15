//#Arch:x86_64
//#Object:other.s
//#LinkArgs:-shared -z now
//#Mode:dynamic
//#ReferenceLinkers:
//#ExpectErrorWild:reference to discarded COMDAT section `.text.missing`

# A live reference to a discarded definition is rejected even in shared output.

.text
.globl use_missing
use_missing:
  jmp missing@PLT
.section .data.first,"awG",@progbits,signature,comdat
.globl first
first:
  .long 42
.section .note.GNU-stack,"",@progbits
