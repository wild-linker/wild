.text
.p2align 2
.globl far_target
.hidden far_target
.type far_target, %function
far_target:
    bti c
    mov w0, #7
    ret
