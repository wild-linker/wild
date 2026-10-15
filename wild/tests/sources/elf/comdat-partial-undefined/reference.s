.text
.ifdef WEAK
.weak missing
.endif
.globl _start
_start:
  call missing
  mov %eax, %edi
  mov $60, %eax
  syscall
.section .note.GNU-stack,"",@progbits
