// jmp *x(%rip), one byte short.
.section .text,"ax",@progbits
.globl _start
.type _start,@function
_start:
  .byte 0xff,0x25,0,0,0
  .reloc _start + 2, R_X86_64_GOTPCRELX, target - 4
.size _start, .-_start
.section .tdata,"awT",@progbits
.globl target
.type target,@tls_object
target:
  .quad 1
.size target, 8
.section .note.GNU-stack,"",@progbits
