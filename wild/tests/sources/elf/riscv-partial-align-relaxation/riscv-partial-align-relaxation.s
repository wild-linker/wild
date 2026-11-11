/*
//#Arch: riscv64
//#CompArgs: -march=rv64gc
//#LinkArgs: -nostdlib -static --relax
//#Relocatable:gate.s
//#ExpectSym:gate
//#ExpectSym:_start size=12
*/

.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    call gate
    li a7, 93
    ecall
.size _start, .-_start
