/*
// GNU ld's default script defines:
//   __global_pointer$ = MIN(__SDATA_BEGIN__ + 0x800,
//                           MAX(__DATA_BEGIN__ + 0x800, __BSS_END__ - 0x800))

//#AbstractConfig:default
//#ReferenceLinkers: bfd
//#Arch: riscv64
//#CompArgs: -march=rv64gc
//#LinkArgs: -nostdlib -static --no-relax
//#RunEnabled: true

// Everything small: gp = __DATA_BEGIN__ + 0x800.
//#Config:data-begin:default
//#CompArgs: -Wa,--defsym,DATA_BEGIN=1

// Large .data pushes .sdata away from the start of .data, and large .bss puts
// __BSS_END__ - 0x800 past .sdata: gp = __SDATA_BEGIN__ + 0x800.
//#Config:sdata-begin:default
//#CompArgs: -Wa,--defsym,SDATA_BEGIN=1

// Large .data, small .bss: __BSS_END__ - 0x800 lies between the other two terms,
// so gp = __BSS_END__ - 0x800.
//#Config:bss-end:default
//#CompArgs: -Wa,--defsym,BSS_END=1
*/

.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    # Reference every section so that none get garbage collected, since their sizes matter.
    lla     t0, data_start
    lla     t1, sdata_start
    lla     t2, sbss_start
    lla     t3, bss_end
    lla     a0, __global_pointer$
.ifdef DATA_BEGIN
    lla     a1, data_start
    sub     a0, a0, a1
    li      a1, 0x800
.endif
.ifdef SDATA_BEGIN
    lla     a1, sdata_start
    sub     a0, a0, a1
    li      a1, 0x800
.endif
.ifdef BSS_END
    lla     a1, bss_end
    sub     a0, a0, a1
    li      a1, -0x800
.endif
    bne     a0, a1, Lfail
    li      a0, 42
    li      a7, 93
    ecall
Lfail:
    li      a0, 1
    li      a7, 93
    ecall
.size _start, .-_start

.section .data, "aw", @progbits
.p2align 3
data_start:
    .quad 1
.ifndef DATA_BEGIN
    .zero 0x2000
.endif

.section .sdata, "aw", @progbits
.p2align 3
sdata_start:
    .quad 2

.section .sbss, "aw", @nobits
.p2align 3
sbss_start:
    .zero 8

.section .bss, "aw", @nobits
.p2align 3
.ifdef SDATA_BEGIN
    .zero 0x2000
.else
    .zero 8
.endif
bss_end:
