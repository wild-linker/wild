.section .rodata,"a",@progbits
.balign 8
.globl read_only_pointer
.type read_only_pointer, @object
read_only_pointer:
  .quad read_only_foo
.size read_only_pointer, .-read_only_pointer

.section .note.GNU-stack,"",@progbits
