.text
.globl _start
.type _start, @function
_start:
    jmp *target@GOTPCREL(%rip)
.size _start, .-_start

.globl target
.hidden target
.type target, @function
target:
    mov $60, %eax
    mov $42, %edi
    syscall
.size target, .-target

.section .note.GNU-stack,"",@progbits
