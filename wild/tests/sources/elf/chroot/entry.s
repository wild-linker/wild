.globl _start
_start:
    mov $60, %eax
    mov $42, %edi
    syscall
.globl discard_me
discard_me:
    ret
.section .note.GNU-stack,"",@progbits
