/*
//#Arch: riscv64
//#CompArgs: -march=rv64gc
//#LinkArgs: -nostdlib -static --relax
//#Relocatable:hidden-call.s
//#ExpectSym:foo size=8
//#ExpectSym:trampoline size=12
//#ExpectSym:_start size=4
*/

.option norvc
.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    call trampoline
.size _start, .-_start
