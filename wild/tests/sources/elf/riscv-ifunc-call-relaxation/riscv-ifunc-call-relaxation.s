/*
//#AbstractConfig:default
//#LinkArgs: -nostdlib -static --relax
//#ReferenceLinkers:bfd
//#ExpectSym:nearby_ifunc
//#RunEnabled:false

//#Config:riscv64:default
//#Arch: riscv64
//#CompArgs: -march=rv64gc
//#ExpectSym:_start size=16

//#Config:riscv32:default
//#Arch: riscv32
//#CompArgs: -march=rv32gc
//#ExpectSym:_start size=16
*/

.section .text, "ax", @progbits
.space 3000

.globl _start
.type _start, @function
_start:
    call nearby_ifunc
    li a7, 93
    ecall
.size _start, .-_start

.globl nearby_ifunc
.type nearby_ifunc, @gnu_indirect_function
nearby_ifunc:
    lla a0, actual_func
    ret
.size nearby_ifunc, .-nearby_ifunc

.type actual_func, @function
actual_func:
    li a0, 42
    ret
.size actual_func, .-actual_func
