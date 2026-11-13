/*
//#Arch:loongarch64
//#LinkArgs:-nostdlib -static --relax
//#LinkerScript:layout.ld
//#RunEnabled:false
//#ExpectSym:_start address=0x10000
//#ExpectSym:far address=0x8010004
//#ExpectSym:_start size=12
*/

.section .text, "ax", @progbits

.globl _start
.type _start, @function
_start:
    call36 near
    # Initially far is 2^27 - 4 bytes ahead. Shrinking the first call moves this
    # call back four bytes, while the linker script keeps far fixed.
    call36 far
.size _start, .-_start

.type near, @function
near:
    ret
.size near, .-near

.section .far, "ax", @progbits
.globl far
.type far, @function
far:
    ret
.size far, .-far
