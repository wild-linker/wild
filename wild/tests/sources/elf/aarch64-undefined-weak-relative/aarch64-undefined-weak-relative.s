// Check that AArch64 PC-relative relocations to an undefined weak symbol
// resolve the symbol to the relocation place, so S - P + A = A.
//
//#Arch:aarch64
//#ReferenceLinkers:lld
//#RunEnabled:false
//#LinkArgs:--image-base=0x10000000 --no-gc-sections
//#ExpectInstructions:.text 0 1: adr x0, 1b
//#ExpectSectionBytes:.text=0x0000009000000090 4..12
//#ExpectInstructions:.text 12 1: ldr x8, 1b
//#ExpectSectionBytes:.data=0x00000000000000000000000000000000
//#DiffIgnore:rel.match_failed.R_AARCH64_PREL16

.weak target

.text
.global _start
_start:
    // R_AARCH64_ADR_PREL_LO21
    adr x0, target

    // R_AARCH64_ADR_PREL_PG_HI21
    adrp x0, target

    // R_AARCH64_ADR_PREL_PG_HI21_NC
    adrp x0, :pg_hi21_nc:target

    // R_AARCH64_LD_PREL_LO19
    ldr x8, target

.data
    // R_AARCH64_PREL32
    .word target - .

    // R_AARCH64_PREL64
    .xword target - .

    // R_AARCH64_PREL16
    .hword target - .

    // Pad to a 4-byte boundary for linker-diff.
    .hword 0
