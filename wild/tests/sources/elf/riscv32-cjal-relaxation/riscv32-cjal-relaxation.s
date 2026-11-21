/*
//#Arch: riscv32
//#CompArgs: -march=rv32gc -mabi=ilp32d
//#LinkArgs: -nostdlib -static --relax
//#ExpectSym:nearby_func
//#ExpectSym:_start size=10
//#ExpectInstructions:.text 0 .short 0x2029; li a7, 93; ecall
*/

.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    call nearby_func
    li a7, 93
    ecall
.size _start, .-_start

.globl nearby_func
.type nearby_func, @function
nearby_func:
    li a0, 42
    ret
.size nearby_func, .-nearby_func
