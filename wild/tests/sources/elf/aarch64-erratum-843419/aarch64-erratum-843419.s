// -Ttext puts the adrp at page offset 0xff8 so the second ldr gets patched, with
// byte 11 being the top byte of the branch. -Tdata keeps value out of adr range,
// otherwise bfd rewrites the adrp to an adr instead of patching.
//#AbstractConfig:base
//#Arch:aarch64
//#Mode:static
//#LinkArgs:--fix-cortex-a53-843419 --no-gc-sections -Ttext=0x200ff8 -Tdata=0x600000
//#ExpectSectionBytes:.text=0x14 11..12
// linker-diff has no model of erratum stubs, so it flags the reference linkers here too.
//#DiffIgnore:rel.match_failed.R_AARCH64_LDST64_ABS_LO12_NC

// bfd puts a trampoline in front of its veneer, so only lld lays the stub out like us.
//#Config:default:base
//#ReferenceLinkers:lld
//#ExpectSectionBytes:.text=0x010040f9fcffff17 24..32

// GNU ld aligns its stub area to 8, but we use 4.
//#Config:bfd:base
//#ReferenceLinkers:bfd
//#DiffIgnore:section.text.alignment

// lld misses this one, it compares q0's register number against x0.
//#Config:simd:bfd
//#CompArgs:-Wa,--defsym,SIMD=1

.text
.globl _start
.type _start, @function
_start:
    adrp x0, value
.ifdef SIMD
    // q0 is a separate register to x0, so should not affect outcome.
    ldr  q0, [x0]
.else
    ldr  x1, [x0]
.endif
    ldr  x1, [x0, :lo12:value]
    mov  x8, #93
    mov  x0, x1
    svc  #0
.size _start, .-_start

.data
.p2align 3
value:
    .quad 42
