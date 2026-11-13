/*
//#Arch:loongarch64
//#LinkArgs:-nostdlib -static --relax
//#LinkerScript:layout.ld
//#RunEnabled:false
//#ExpectSym:_start address=0x10000
//#ExpectSym:near size=12
//#ExpectSym:far address=0x8010004
//#ExpectSym:_start size=4
//#ExpectSym:near address=0x10004
*/

.section .text.start, "ax", @progbits
.globl _start
.type _start, @function
_start:
    call36 near
.size _start, .-_start

.section .text.near, "ax", @progbits
.globl near
.type near, @function
near:
    # Shrinking .text.start moves this entire input section back four bytes.
    # With far fixed, the displacement grows from 2^27 - 4 to 2^27.
    call36 far
    ret
.size near, .-near

.section .far, "ax", @progbits
.globl far
.type far, @function
far:
    ret
.size far, .-far
