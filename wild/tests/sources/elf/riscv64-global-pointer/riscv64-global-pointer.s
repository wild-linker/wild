/*
//#ReferenceLinkers: bfd
//#Arch: riscv64
//#CompArgs: -march=rv64gc
//#LinkArgs: -nostdlib -static --no-relax
//#RunEnabled: true
*/

.section .text, "ax", @progbits
.globl _start
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

.data
data_start:
    .word 1
