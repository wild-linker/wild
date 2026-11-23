// A TLSDESC_CALL marker at the end of the section.
.section .text,"ax",@progbits
.globl _start
.type _start,@function
_start:
  .byte 0xff
  .reloc _start + 1, R_X86_64_TLSDESC_CALL, target 
.size _start, .-_start
.section .tdata,"awT",@progbits
.globl target
.type target,@tls_object
target:
  .quad 1
.size target, 8
.section .note.GNU-stack,"",@progbits
