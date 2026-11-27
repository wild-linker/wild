//#Arch:x86_64
//#CompArgs:-Wa,-L
//#LinkArgs:-static --emit-relocs --no-gc-sections
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectReloc:.debug_info offset=0,target=.debug_abbrev,type=1,addend=0
//#ExpectSym:.Lunused

.globl _start
_start:
.Lunused:
    ret

.section .debug_info,"",@progbits
    .quad .debug_abbrev

.section .debug_abbrev,"",@progbits
    .byte 1

.section .note.GNU-stack,"",@progbits
