// Nonzero addends must not turn a page-relative address into a different ADR target.
//#Arch:aarch64
//#Mode:static
//#ReferenceLinkers:bfd,lld
//#LinkArgs:--no-gc-sections -Ttext=0x201000 -Tdata=0x210000
//#RunEnabled:false
//#ExpectInstructions:.text 4 add x0, x0, #0
//#ExpectInstructions:.text 12 add x1, x1, #0
//#ExpectInstructions:.text 20 add x2, x2, #0
//#ExpectInstructions:.text 28 add x3, x3, #0

.text
.globl _start
.type _start, @function
_start:
    adrp x0, target
    add x0, x0, :lo12:target+4096
    adrp x1, target
    add x1, x1, :lo12:target-4096
    adrp x2, target+4096
    add x2, x2, :lo12:target
    adrp x3, target+4096
    add x3, x3, :lo12:target+4096
    ret
.size _start, .-_start

.data
.globl target
target:
    .quad 42
