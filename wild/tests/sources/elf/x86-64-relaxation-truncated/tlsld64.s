// lea, then just the start of a 64-bit movabs.
.section .text,"ax",@progbits
.globl _start
.type _start,@function
_start:
  .byte 0x48,0x8d,0x3d,0,0,0,0,0x48,0xb8
  .reloc _start + 3, R_X86_64_TLSLD, target - 4
.size _start, .-_start
.section .tdata,"awT",@progbits
.globl target
.type target,@tls_object
target:
  .quad 1
.size target, 8
.section .note.GNU-stack,"",@progbits
