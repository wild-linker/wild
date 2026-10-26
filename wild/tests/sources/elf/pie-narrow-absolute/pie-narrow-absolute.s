//#AbstractConfig:default
//#Arch:x86_64
//#Compiler:gcc
//#LinkArgs:-pie --no-dynamic-linker --no-gc-sections
//#ReferenceLinkers:bfd,lld
//#ExpectError:R_X86_64_32
//#ExpectErrorWild:R_X86_64_32.*cannot use a runtime address relocation.*full 8-byte address field

//#Config:ordinary:default
//#CompArgs:-Wa,--defsym,ifunc=0

//#Config:ordinary-relr:ordinary
//#LinkArgs:-z pack-relative-relocs

//#Config:ifunc:default
//#CompArgs:-Wa,--defsym,ifunc=1

//#Config:ifunc-relr:ifunc
//#LinkArgs:-z pack-relative-relocs

.text
.globl _start
_start:
    ret

.if ifunc
.type foo, @gnu_indirect_function
.else
.type foo, @function
.endif
foo:
    lea impl(%rip), %rax
    ret
impl:
    ret

.data
.long foo
.long 0

.section .note.GNU-stack,"",@progbits
