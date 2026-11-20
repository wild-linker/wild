/*
//#AbstractConfig:default
//#ReferenceLinkers:
//#ExpectError:'f'.*incompatible.*'zfinx'
//#LinkArgs:-no-pie

//#Config:riscv64:default
//#Arch: riscv64
//#CompArgs:-march=rv64imafc -mabi=lp64
//#Object:riscv-attr-conflict-1.s:-march=rv64imac_zfinx -mabi=lp64

//#Config:riscv32:default
//#Arch: riscv32
//#CompArgs:-march=rv32imafc -mabi=ilp32
//#Object:riscv-attr-conflict-1.s:-march=rv32imac_zfinx -mabi=ilp32
*/

.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    li a0, 42
    li a7, 93
    ecall
.size _start, .-_start
