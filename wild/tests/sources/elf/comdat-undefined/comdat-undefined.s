//#Arch:x86_64
//#Object:other.s
//#ExpectError:missing
//#RunEnabled:false

.text
.globl _start
_start:
  call missing

.section .data.first,"awG",@progbits,signature,comdat
.globl first
first:
  .long 42
.section .note.GNU-stack,"",@progbits
