.section .data.foo,"awG",@progbits,signature,comdat
.globl foo
foo:
  .long 99

.section .data.losing_only,"awG",@progbits,signature,comdat
.globl losing_only
losing_only:
  .long 31

.section .note.GNU-stack,"",@progbits
