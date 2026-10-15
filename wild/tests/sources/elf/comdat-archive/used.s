.text
.globl helper
helper:
  ret
.section .data.foo,"awG",@progbits,signature,comdat
.globl foo, losing_only
foo:
losing_only:
  .long 99
.section .note.GNU-stack,"",@progbits
