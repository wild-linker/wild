/*
//#AbstractConfig:default
//#LinkArgs: -nostdlib -static --gc-sections
//#NoSym:unused_func
//#ExpectSym:_start
//#ExpectSectionBytes:.debug_test=0x0804

//#Config:riscv64:default
//#Arch: riscv64
//#CompArgs: -march=rv64gc

//#Config:riscv32:default
//#Arch: riscv32
//#CompArgs: -march=rv32gc
*/

.section .text._start,"ax",@progbits
.globl _start
.type _start, @function
_start:
.LFB0:
.LVL0:
        li      a7, 93
.LVL1:
        li      a0, 42
.LVL2:
        ecall
.LFE0:
        .size _start, .-_start
        .section .text.unused_func,"ax",@progbits
        .globl unused_func
        .type unused_func, @function

unused_func:
.LFB1:
        ret
.LFE1:
        .size unused_func, .-unused_func
        .section .debug_aranges,"",@progbits
        .8byte  .LFB0
        .8byte  .LFE0 - .LFB0
        .8byte  .LFB1
        .8byte  .LFE1 - .LFB1

        /* .LVL2 - .LVL0 = 8, .LVL1 - .LVL0 = 4: expect bytes 08 04 */
        .section .debug_test,"",@progbits
        .uleb128 .LVL2 - .LVL0
        .uleb128 .LVL1 - .LVL0
