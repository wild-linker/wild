/*
//#Arch:riscv64
//#CompArgs:-march=rv64gc
//#LinkArgs:-static --emit-relocs --relax
//#Object:value.s
//#ReferenceLinkers:bfd
//#RunEnabled:false
//#DiffEnabled:false
// BFD deletes LUI and rewrites R_RISCV_LO12_I/S to 69/70.
//#ExpectSym:_start size=10
//#ExpectReloc:.text at=_start,target=abs_sym,type=69,addend=0
//#ExpectReloc:.text offset=4,target=abs_sym,type=70,addend=0
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
