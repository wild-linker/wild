//#Arch:x86_64
//#LinkArgs:-static --emit-relocs --no-gc-sections
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectSection:.rela.debug_info
//#ExpectReloc:.debug_info offset=0,target=_start,type=1,addend=0

.globl _start
_start:
    ret

.section .debug_info,"",@progbits
    .quad _start

.section .note.GNU-stack,"",@progbits
