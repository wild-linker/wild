// Check that AArch64 branches to an undefined weak symbol resolve
// to the next instruction in static output and to the PLT in shared output.
//
//#AbstractConfig:default
//#Arch:aarch64
//#ReferenceLinkers:lld
//#RunEnabled:false
//#LinkArgs:--image-base=0x10000000 --no-gc-sections
//#Config:static:default
//#ExpectSectionBytes:.text=0x01000014010000942000005421000036c0035fd6

//#Config:shared:default
//#Mode:dynamic
//#LinkArgs:-shared
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT
//#DiffIgnore:section.got.plt.entsize

//#Config:shared-conditional-only:shared
//#CompArgs:-Wa,--defsym,CONDITIONAL_ONLY=1

.weak target

.text
.global _start
_start:
.ifndef CONDITIONAL_ONLY
    // R_AARCH64_JUMP26
    b target

    // R_AARCH64_CALL26
    bl target
.endif

    // R_AARCH64_CONDBR19
    b.eq target

    // R_AARCH64_TSTBR14
    tbz x1, #0, target

    ret
