/*
//#AbstractConfig:default
//#LinkArgs: -nostdlib -static --relax
//#Relocatable:hidden-call.s
//#ExpectSym:foo size=8

//#Config:riscv64:default
//#Arch: riscv64
//#CompArgs: -march=rv64gc
//#ExpectSym:trampoline size=12
//#ExpectSym:_start size=4

//#Config:riscv32:default
//#Arch: riscv32
//#CompArgs: -march=rv32gc
//#ExpectSym:trampoline size=10
//#ExpectSym:_start size=2
*/

.option norvc
.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    call trampoline
.size _start, .-_start
