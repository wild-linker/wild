/*
//#Arch:riscv64
//#CompArgs:-march=rv64gc
//#LinkArgs:-static --emit-relocs --relax
//#Object:value.s
// LLD keeps the full-width LUI, matching the retained R_RISCV_HI20 relocation.
//#ReferenceLinkers:lld
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectSym:_start size=10
//#ExpectReloc:.text at=_start,target=abs_sym,type=26,addend=0
//#ExpectReloc:.text offset=4,target=abs_sym,type=27,addend=0
*/

.option relax
.globl _start
.type _start, @function
_start:
    lui a0, %hi(abs_sym)
    addi a0, a0, %lo(abs_sym)
    ret
.size _start, .-_start

.section .note.GNU-stack,"",@progbits
