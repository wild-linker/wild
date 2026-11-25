/*
//#AbstractConfig:default
//#ReferenceLinkers:bfd,lld
//#LinkArgs: -nostdlib -static --emit-relocs
//#ExpectSection:.rela.text
//#ExpectSym:second_call
//#ExpectReloc:.text at=second_call,target=nearby_func

//#Config:no-relax:default
//#Arch:riscv64,riscv32
//#LinkArgs: --no-relax
//#ExpectSym:_start size=24
// R_RISCV_CALL_PLT
//#ExpectReloc:.text at=_start,target=nearby_func,type=19,addend=0

//#Config:riscv64-relax:default
//#Arch: riscv64
//#CompArgs: -march=rv64gc
//#LinkArgs: --relax
//#ExpectSym:_start size=20
// R_RISCV_JAL
//#ExpectReloc:.text at=_start,target=nearby_func,type=17,addend=0

//#Config:riscv32-relax:default
//#Arch: riscv32
//#CompArgs: -march=rv32gc
//#LinkArgs: --relax
//#ExpectSym:_start size=18
// R_RISCV_RVC_JUMP
//#ExpectReloc:.text at=_start,target=nearby_func,type=45,addend=0
*/

.option relax
.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    call nearby_func
.option norelax
.globl second_call
second_call:
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
