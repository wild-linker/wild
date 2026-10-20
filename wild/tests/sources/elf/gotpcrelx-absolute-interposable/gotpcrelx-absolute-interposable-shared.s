.text
.globl load32
.type load32,@function
load32:
    mov target@GOTPCREL(%rip), %eax
    ret

.globl load64
.type load64,@function
load64:
    mov target@GOTPCREL(%rip), %rax
    ret

.globl subtract
.type subtract,@function
subtract:
    mov $42, %eax
    sub target@GOTPCREL(%rip), %rax
    ret

.globl compare
.type compare,@function
compare:
    mov $42, %eax
    cmp target@GOTPCREL(%rip), %rax
    sete %al
    movzbl %al, %eax
    ret

.globl target
.set target, 7
.section .note.GNU-stack,"",@progbits
