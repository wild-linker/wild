//#Config:no-relax
//#Arch:aarch64
//#Mode:static
//#ReferenceLinkers:bfd,lld
//#LinkArgs:-z noexecstack --no-relax

.globl _start
.type _start, @function
_start:
    adrp x0, :got:target
    ldr d0, [x0, :got_lo12:target]
    fmov x0, d0
    ldr x0, [x0]
    mov x8, #93
    svc #0
.size _start, .-_start

.data
.globl target
.hidden target
target:
    .quad 42
