.section .data.foo,"awG",@progbits,signature,comdat
.globl foo
foo:
  .long 42
.section .data.ptr,"awG",@progbits,signature,comdat
.globl ptr
ptr:
  .quad foo
.section .bss.foo,"awG",@nobits,signature,comdat
.globl bss
bss:
  .zero 4
.section .rodata.str,"aMSG",@progbits,1,signature,comdat
.asciz "group string"
.section .note.GNU-stack,"",@progbits
