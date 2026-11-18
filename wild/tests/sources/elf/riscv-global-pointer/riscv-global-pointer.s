/*
 * checking that __global_pointer$ is set the same in symtab and relocations: .data + 0x800
//#AbstractConfig:default
//#ReferenceLinkers: bfd
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
    lla     a1, data_start
    sub     a0, a0, a1
    li      a1, 0x800
    bne     a0, a1, Lfail
    li      a0, 42
    li      a7, 93
    ecall
Lfail:
    li      a0, 1
    li      a7, 93
    ecall
.size _start, .-_start

.section .data, "aw", @progbits
.p2align 3
data_start:
    .quad 1

