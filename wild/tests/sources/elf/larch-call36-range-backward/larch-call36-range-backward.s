/*
//#Arch:loongarch64
//#LinkArgs:-nostdlib -static --relax
//#LinkerScript:layout.ld
//#RunEnabled:false
//#ExpectSym:_start address=0x8010008,size=8
//#ExpectSym:target size=4
//#ExpectSym:helper size=4
//#ExpectSym:target address=0x10004
*/

.section .text.low, "ax", @progbits
.globl helper
.type helper, @function
helper:
    call36 target
.size helper, .-helper

.globl target
.type target, @function
target:
    ret
.size target, .-target

.section .text.high, "ax", @progbits
.globl _start
.type _start, @function
_start:
    # Shrinking helper moves target back four bytes while this call stays fixed.
    # The displacement changes from -2^27 to -2^27 - 4.
    call36 target
.size _start, .-_start
