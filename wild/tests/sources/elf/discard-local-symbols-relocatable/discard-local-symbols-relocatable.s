//#AbstractConfig:default
//#Arch:x86_64
//#CompArgs:-Wa,-L
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectSym:used section=.text.used
//#ExpectSym:.Lused section=.text.used

//#Config:keep-default:default
//#LinkArgs:-r
//#ExpectSym:unused section=.text.used
//#ExpectSym:.Lunused section=.text.used

//#AbstractConfig:discard-all:default
//#NoSym:unused
//#NoSym:.Lunused
//#ExpectSym:optional

//#Config:discard-all-after-r:discard-all
//#LinkArgs:-r -x

//#Config:discard-all-before-r:discard-all
//#LinkArgs:--discard-all -r

//#AbstractConfig:discard-locals:default
//#ExpectSym:unused section=.text.used
//#NoSym:.Lunused
//#ExpectSym:optional

//#Config:discard-locals-after-r:discard-locals
//#LinkArgs:-r -X

//#Config:discard-locals-before-r:discard-locals
//#LinkArgs:--discard-locals -r

.text
.globl _start
.weak optional
_start:
    call optional
    # Explicit relocations prevent the assembler from substituting section symbols.
    .byte 0xe8
    .reloc ., R_X86_64_PLT32, used-4
    .long 0
    .byte 0xe8
    .reloc ., R_X86_64_PLT32, .Lused-4
    .long 0
    ret

.section .text.used,"ax",@progbits
used:
.Lused:
    ret
unused:
.Lunused:
    nop
