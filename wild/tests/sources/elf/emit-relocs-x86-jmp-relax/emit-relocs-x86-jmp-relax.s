/*
//#Arch:x86_64
//#LinkArgs:-static --emit-relocs
//#ReferenceLinkers:bfd
//#RunEnabled:false
//#DiffEnabled:false
// BFD shifts the relaxed jmp displacement offset by -1 and emits R_X86_64_PC32.
//#ExpectReloc:.text offset=1,target=target,type=2,addend=-4
*/

.text
.globl _start
.type _start, @function
_start:
    jmp *target@GOTPCREL(%rip)
.size _start, .-_start

.globl target
.type target, @function
target:
    ret
.size target, .-target

.section .note.GNU-stack,"",@progbits
