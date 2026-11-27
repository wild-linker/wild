/*
//#Arch:riscv64
//#LinkArgs:-static --emit-relocs --gc-sections --no-relax
// LLD replaces the debug-only local symbol relocation with R_RISCV_NONE.
//#ReferenceLinkers:bfd
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectReloc:.debug_info offset=0,target=.Ldebug_only,type=2,addend=0
//#NoSym:dead
*/

.section .text,"ax",@progbits
.globl _start
_start:
    nop
.Ldebug_only:
    ret

.section .text.dead,"ax",@progbits
.globl dead
dead:
    ret

.section .debug_info,"",@progbits
    .quad .Ldebug_only
    .quad dead

.section .note.GNU-stack,"",@progbits
