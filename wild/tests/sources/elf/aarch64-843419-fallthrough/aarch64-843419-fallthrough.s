//#AbstractConfig:default
//#Arch:aarch64
//#ReferenceLinkers:bfd,lld
//#Object:tail.s
//#DiffEnabled:false

//#Config:without-fix:default

//#Config:with-fix:default
//#LinkArgs:--fix-cortex-a53-843419

.text
.global _start
_start:
    bl _init
    mov x0, 42
    mov x8, 93
    svc 0

// The next .init input section completes this function by returning to _start.
.section .init,"ax",@progbits
.p2align 2
.global _init
_init:
    adrp x0, datum
    ldr x1, [sp]
    ldr x2, [x0, :lo12:datum]

.data
.p2align 3
datum:
    .quad 0
