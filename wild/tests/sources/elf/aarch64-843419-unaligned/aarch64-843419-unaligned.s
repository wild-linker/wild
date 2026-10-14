//#AbstractConfig:default
//#Arch:aarch64
//#ReferenceLinkers:bfd,lld
//#LinkArgs:--no-relax --fix-cortex-a53-843419 -T ./layout.ld
//#DiffEnabled:false

//#Config:one-byte:default
//#CompArgs:-Wa,--defsym,TRAILING_BYTES=1

//#Config:two-bytes:default
//#CompArgs:-Wa,--defsym,TRAILING_BYTES=2

//#Config:three-bytes:default
//#CompArgs:-Wa,--defsym,TRAILING_BYTES=3

.text
.p2align 2
.global _start
_start:
    // The linker script places this ADRP at page offset 0xff8.
    adrp x0, datum
    ldr x1, [sp]
    ldr x2, [x0, :lo12:datum]
    mov x0, 42
    mov x8, 93
    svc 0

// Executable sections can end with data that is not instruction-aligned.
.fill TRAILING_BYTES, 1, 0x5a

.data
.p2align 3
datum:
    .quad 0
