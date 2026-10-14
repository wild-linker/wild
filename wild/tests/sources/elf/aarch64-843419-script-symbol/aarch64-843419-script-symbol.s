//#AbstractConfig:default
//#Arch:aarch64
//#ReferenceLinkers:bfd,lld
//#LinkArgs:--no-gc-sections -T ./layout.ld
//#RunEnabled:false
//#DiffEnabled:false

//#Config:without-fix:default

//#Config:with-fix:default
//#LinkArgs:--fix-cortex-a53-843419

.text
.p2align 2
.global _start
_start:
    adrp x0, 0
    ldr x1, [x2]
    ldr x3, [x0]
    ret

.data
    .quad 0
