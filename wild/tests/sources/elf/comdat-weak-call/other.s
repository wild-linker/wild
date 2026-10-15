.text
.global _start
_start:
  bl missing
  mov x0, #42
  mov x8, #93
  svc #0

.section .text.missing,"axG",@progbits,signature,comdat
.weak missing
missing:
  ret
.section .note.GNU-stack,"",@progbits
