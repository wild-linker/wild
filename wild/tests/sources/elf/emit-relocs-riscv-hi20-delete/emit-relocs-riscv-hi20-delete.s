/*
//#Arch:riscv64
//#CompArgs:-march=rv64gc
//#LinkArgs:-static --emit-relocs --relax
//#Object:value.s
//#ReferenceLinkers:
//#RunEnabled:false
//#DiffEnabled:false
// Wild disables LUI deletion under --emit-relocs, retaining standard HI20/LO12 relocations.
//#ExpectSym:_start size=18
//#ExpectReloc:.text at=_start,target=abs_sym,type=26,addend=0
//#ExpectReloc:.text offset=4,target=abs_sym,type=27,addend=0
//#ExpectReloc:.text offset=8,target=abs_sym,type=26,addend=0
//#ExpectReloc:.text offset=12,target=abs_sym,type=28,addend=0
*/

.option relax
.globl _start
.type _start, @function
_start:
    lui a0, %hi(abs_sym)
    addi a0, a0, %lo(abs_sym)
    lui a1, %hi(abs_sym)
    sw a0, %lo(abs_sym)(a1)
    ret
.size _start, .-_start

.section .note.GNU-stack,"",@progbits
