/*
//#Arch:loongarch64
//#LinkArgs:-nostdlib -static
//#LinkerScript:layout.ld
//#RunEnabled:false
//#ExpectSym:_start size=8
//#ExpectSym:outside address=0x10000000
*/

.section .text, "ax", @progbits

.globl _start
.type _start, @function
_start:
    call36 outside
.size _start, .-_start

.section .far, "ax", @progbits
.globl outside
.type outside, @function
outside:
    ret
.size outside, .-outside
