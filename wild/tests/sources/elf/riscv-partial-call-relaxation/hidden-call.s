.option norvc
.section .text, "ax", @progbits

.hidden foo
.globl foo
.type foo, @function
foo:
    li a0, 42
    ret
.size foo, .-foo

.globl trampoline
.type trampoline, @function
trampoline:
    call foo
    li a7, 93
    ecall
.size trampoline, .-trampoline
