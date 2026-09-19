// ldr then madd gets the madd patched, with byte 15 being the top byte of the branch.
// The madd mustn't use the loaded register, since a dependent one is safe and bfd
// skips it. Note that lld doesn't implement this one at all.
//#AbstractConfig:base
//#Arch:aarch64
//#Mode:static
//#LinkArgs:--fix-cortex-a53-835769 --no-gc-sections -Ttext=0x200000
//#ExpectSectionBytes:.text=0x14 15..16

// bfd puts a trampoline in front of its veneer, so only wild can check the stub itself.
//#Config:default:base
//#ReferenceLinkers:
//#ExpectSectionBytes:.text=0x1f2003d5620c039bfbffff17 28..40

// GNU ld aligns its stub area to 8, but we use 4.
//#Config:bfd:base
//#ReferenceLinkers:bfd
//#DiffIgnore:section.text.alignment
//#DiffIgnore:literal-byte-mismatch

//#Config:regions
//#RunEnabled:false
//#Arch:aarch64
//#Mode:static
//#LinkerScript:regions.ld
//#DiffEnabled:false
//#ExpectSym:value address=0x300000
//#LinkArgs:--fix-cortex-a53-835769 --no-gc-sections
//#ReferenceLinkers:bfd
//#ExpectSectionBytes:.text=0x14 15..16

.text
.globl _start
.type _start, @function
_start:
    mov  x3, #6
    adrp x0, value
    ldr  x1, [x0, :lo12:value]
    madd x2, x3, x3, x3
    mov  x8, #93
    mov  x0, x2
    svc  #0
.size _start, .-_start

.data
.p2align 3
value:
    .quad 6
