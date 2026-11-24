/*
//#AbstractConfig:default
//#ReferenceLinkers: lld
//#LinkArgs: -nostdlib -static --no-relax
//#RunEnabled: true

//#Config:riscv64:default
//#Arch: riscv64
//#CompArgs: -march=rv64gc

//#Config:riscv32:default
//#Arch: riscv32
//#CompArgs: -march=rv32gc
*/

.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    lla     a0, __global_pointer$
    lla     a1, sdata_start
    sub     a0, a0, a1
    li      a2, 0x800
    bne     a0, a2, Lfail
    li      a0, 42
    li      a7, 93
    ecall
Lfail:
    li      a0, 1
    li      a7, 93
    ecall
.size _start, .-_start

.section .sdata, "aw", @progbits
.p2align 3
sdata_start:
    .quad 1
