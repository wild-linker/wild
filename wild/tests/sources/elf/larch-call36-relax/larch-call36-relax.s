/*
//#AbstractConfig:common
//#Arch:loongarch64
//#LinkArgs:-nostdlib -static

//#Config:relax:common
//#LinkArgs:--relax
//#ExpectSym:_start size=36
//#ExpectSym:helper size=4
//#ExpectSym:leaf size=8

//#Config:no-relax:common
//#LinkArgs:--no-relax
//#ExpectSym:_start size=40
//#ExpectSym:helper size=8
//#ExpectSym:leaf size=8
*/

.section .text, "ax", @progbits

.globl _start
.type _start, @function
_start:
    call36 helper
    addi.w $r12, $r0, 42
    bne $r4, $r12, .Lfail
    addi.w $r4, $r0, 42
    addi.w $r11, $r0, 93
    syscall 0
.Lfail:
    addi.w $r4, $r0, 1
    addi.w $r11, $r0, 93
    syscall 0
.size _start, .-_start

# Tail call must become `b`, preserving $ra so leaf returns to _start.
.globl helper
.type helper, @function
helper:
    tail36 $t0, leaf
.size helper, .-helper

.globl leaf
.type leaf, @function
leaf:
    addi.w $r4, $r0, 42
    jirl $r0, $r1, 0
.size leaf, .-leaf
