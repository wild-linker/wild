//#Arch:x86_64
//#LinkArgs:-static --emit-relocs --no-gc-sections
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectSection:.rodata size=10
//#ExpectReloc:.data at=ptr,target=.rodata,type=1,addend=0

.globl _start
_start:
    ret

.section .rodata.str1.1,"aMS",@progbits,1
    .asciz "duplicate"
.Lsecond:
    .asciz "duplicate"

.section .data,"aw",@progbits
.globl ptr
ptr:
    .quad .Lsecond

.section .note.GNU-stack,"",@progbits
