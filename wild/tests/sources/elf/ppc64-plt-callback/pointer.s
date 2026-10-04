.section .rodata,"a",@progbits
.balign 8
.globl callback
.type callback, @object
callback:
  .quad selected
.size callback, .-callback

.section .note.GNU-stack,"",@progbits
