/*
//#Arch:loongarch64
//#LinkArgs:-nostdlib -static --no-gc-sections --relax
//#LinkerScript:layout.ld
//#RunEnabled:false
//#ExpectSym:_start address=0x10000
//#ExpectSym:caller size=8
//#ExpectSym:far address=0x8010018
//#ExpectSym:_start size=4
//#ExpectSym:caller address=0x10010
*/

# Set section alignment without emitting R_LARCH_ALIGN.
.section .text.start, "ax", @progbits
.option push
.option norelax
.p2align 4
.option pop

.globl _start
.type _start, @function
_start:
    call36 near
.size _start, .-_start
near:
    ret
    nop
    nop

.section .text.caller, "ax", @progbits
.option push
.option norelax
.p2align 4
.option pop

.globl caller
.type caller, @function
caller:
    # Shrinking .text.start from 20 to 16 bytes also removes 12 bytes of padding.
    # This call moves back 16 bytes, taking the fixed target out of B26 range.
    call36 far
.size caller, .-caller

.section .far, "ax", @progbits
.globl far
.type far, @function
far:
    ret
.size far, .-far
