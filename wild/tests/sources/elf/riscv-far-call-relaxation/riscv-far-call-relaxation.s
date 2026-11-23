/*
//#AbstractConfig:default
//#LinkArgs: -nostdlib -static --relax
//#ExpectSym:far size=4

//#Config:riscv64:default
//#Arch: riscv64
//#CompArgs: -march=rv64gc
//#ExpectSym:_start size=100

//#Config:riscv32:default
//#Arch: riscv32
//#CompArgs: -march=rv32gc
//#ExpectSym:_start size=60
*/

.option norvc
.option relax
.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    .rept 20
    call near
    .endr
    call far
    li a0, 42
    li a7, 93
    ecall
.size _start, .-_start

near:
    ret
    .zero 1048628
.globl far
.type far, @function
far:
    ret
.size far, .-far
