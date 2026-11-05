//#AbstractConfig:base
//#Arch:aarch64
//#Mode:static
//#ReferenceLinkers:bfd,lld
// Keep target outside ADR range so LLD uses adrp/add rather than nop/adr.
//#LinkArgs:-z noexecstack -Ttext=0x500000 -Tdata=0x5000000

//#Config:relaxed:base

//#Config:no-relax:base
//#LinkArgs:--no-relax

.globl small_absolute
.hidden small_absolute
.set small_absolute, 2
.globl large_absolute
.hidden large_absolute
.set large_absolute, 0x12345678

.data
.globl target
.hidden target
target:
    .quad 42

.text
.globl _start
.type _start, @function
_start:
    adrp x0, :got:small_absolute
    ldr x0, [x0, :got_lo12:small_absolute]
    cmp x0, #2
    b.ne failed

    adrp x9, :got:large_absolute
    ldr x9, [x9, :got_lo12:large_absolute]
    mov x10, #0x5678
    movk x10, #0x1234, lsl #16
    cmp x9, x10
    b.ne failed

    adrp x19, :got:target
    ldr x19, [x19, :got_lo12:target]
    ldr x0, [x19]
    cmp x0, #42
    b.ne failed

    mov x0, #42
    b exit
failed:
    mov x0, #43
exit:
    mov x8, #93
    svc #0
.size _start, .-_start
