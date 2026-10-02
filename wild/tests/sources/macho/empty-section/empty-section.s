//#Object:runtime.c
//#ExpectSection:__empty
//#ExpectSectionBytes:__empty=0x
//#ExpectSym:_empty section="__empty",segment="__DATA",offset-in-section=0

// A reference must load this section even though it contains no bytes.
.section __DATA,__empty
.globl _empty
_empty:

.text
.p2align 2
.globl _main
_main:
    adrp x0, _empty@PAGE
    add x0, x0, _empty@PAGEOFF
    mov x0, #42
    b _exit_syscall
