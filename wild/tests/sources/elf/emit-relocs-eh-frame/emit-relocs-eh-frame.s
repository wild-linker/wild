//#AbstractConfig:default
//#Arch:x86_64
//#LinkArgs:-static --emit-relocs
//#RunEnabled:false
//#DiffEnabled:false

//#Config:no-gc:default
//#ReferenceLinkers:bfd,lld
//#LinkArgs:--no-gc-sections
//#ExpectReloc:.eh_frame offset=0x20,target=.text,type=2,addend=0
//#ExpectReloc:.eh_frame offset=0x34,target=.text,type=2,addend=1

//#Config:gc:default
// LLD omits .eh_frame relocations with --gc-sections; BFD retains the live relocation.
//#ReferenceLinkers:bfd
//#LinkArgs:--gc-sections
//#NoSym:dead
//#ExpectReloc:.eh_frame offset=0x20,target=.text,type=2,addend=0

.section .text.dead,"ax",@progbits
dead:
    .cfi_startproc
    ret
    .cfi_endproc

.section .text.live,"ax",@progbits
.globl _start
_start:
    .cfi_startproc
    ret
    .cfi_endproc

.section .note.GNU-stack,"",@progbits
