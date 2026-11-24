/*
//#AbstractConfig:default
//#ReferenceLinkers: lld
//#LinkArgs: -nostdlib -shared --no-relax
//#RunEnabled: false
//#NoSym: __global_pointer$
//#NoDynSym: __global_pointer$

//#Config:riscv64:default
//#Arch: riscv64
//#CompArgs: -march=rv64gc -fPIC

//#Config:riscv32:default
//#Arch: riscv32
//#CompArgs: -march=rv32gc -fPIC
*/

.section .text, "ax", @progbits
.globl shared_fn
.type shared_fn, @function
shared_fn:
    lla     a0, shared_sdata
    ret
.size shared_fn, .-shared_fn

.section .sdata, "aw", @progbits
.p2align 3
shared_sdata:
    .quad 3
