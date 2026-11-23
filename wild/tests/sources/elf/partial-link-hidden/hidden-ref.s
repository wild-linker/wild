.section .text.use_referenced_hidden,"ax",@progbits
.globl use_referenced_hidden
use_referenced_hidden:
  mov referenced_hidden(%rip), %eax
  ret
.hidden referenced_hidden
.section .note.GNU-stack,"",@progbits
