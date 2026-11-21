/*
//#AbstractConfig:default
//#LinkArgs: -nostdlib -static --relax
//#ExpectSym:nearby_func

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
