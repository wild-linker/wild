//#AbstractConfig:base
//#Arch:aarch64
//#Mode:static
//#ReferenceLinkers:bfd,lld
//#DiffEnabled:true
//#LinkArgs:-z noexecstack
//#DiffIgnore:rel.missing-opt.R_AARCH64_TLSLE_ADD_TPREL_HI12.ReplaceWithNop.*

//#Config:relaxed:base

//#Config:no-relax:base
//#LinkArgs:--no-relax

.section .tdata,"awT",@progbits
.balign 8
small:
    .quad 42
    .zero 8192
large:
    .quad 43

.text
.globl _start
.type _start, @function
_start:
    mov x0, #0
    add x0, x0, :tprel_hi12:small
    add x0, x0, :tprel_lo12_nc:small
    cmp x0, #16
    b.ne failed

    mov x1, #0
    add x1, x1, :tprel_hi12:large
    add x1, x1, :tprel_lo12_nc:large
    mov x2, #0x2018
    cmp x1, x2
    b.ne failed

    mov x0, #42
    b exit
failed:
    mov x0, #43
exit:
    mov x8, #93
    svc #0
.size _start, .-_start
