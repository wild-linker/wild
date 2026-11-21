/*
//#AbstractConfig:default
//#LinkArgs: -nostdlib -static --relax
//#Relocatable:gate.s
//#ExpectSym:gate

//#Config:riscv64:default
//#Arch: riscv64
//#CompArgs: -march=rv64gc
//#ExpectSym:_start size=12

//#Config:riscv32:default
//#Arch: riscv32
//#CompArgs: -march=rv32gc
//#ExpectSym:_start size=10
*/

.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    call gate
    li a7, 93
    ecall
.size _start, .-_start
