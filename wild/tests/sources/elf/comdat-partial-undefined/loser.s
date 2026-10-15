.section .text.missing,"axG",@progbits,signature,comdat
.ifdef WEAK
.weak missing
.else
.globl missing
.endif
.type missing,@function
missing:
  mov $99, %eax
  ret
.section .note.GNU-stack,"",@progbits
