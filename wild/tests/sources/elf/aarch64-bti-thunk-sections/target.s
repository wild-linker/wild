.section .text.big,"ax",@progbits
.p2align 2
.globl big_anchor
big_anchor:
    .space 0x08000000

.section .text.small,"ax",@progbits
.p2align 2
.globl far_target
.hidden far_target
.type far_target, %function
far_target:
    mov w0, #7
    ret
