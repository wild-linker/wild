.option rvc
.section .text, "ax", @progbits
.globl gate
.type gate, @function
gate:
    .option push
    .option norvc
    addi zero, zero, 0
    .option pop
    .p2align 3
    li a0, 42
    ret
.size gate, .-gate
