// Check that AArch64 MOVW PC-relative relocations to an undefined weak symbol
// resolve the symbol to the relocation place, so S - P + A = A.
//
// LLD currently resolves the G0 variants to 4, while BFD resolves them to 0,
// matching the AArch64 undefined-weak PC-relative semantics.
//
//#Arch:aarch64
//#ReferenceLinkers:bfd
//#RunEnabled:false
//#LinkArgs:--no-gc-sections
//#ExpectInstructions:.text 0 movz x1, #0; movk x2, #0; movz x3, #0, lsl #16; movk x4, #0, lsl #16; movz x5, #0, lsl #32; movk x6, #0, lsl #32; movz x7, #0, lsl #48

.weak target

.text
.global _start
_start:
    // R_AARCH64_MOVW_PREL_G0
    movz x1, #:prel_g0:target

    // R_AARCH64_MOVW_PREL_G0_NC
    movk x2, #:prel_g0_nc:target

    // R_AARCH64_MOVW_PREL_G1
    movz x3, #:prel_g1:target

    // R_AARCH64_MOVW_PREL_G1_NC
    movk x4, #:prel_g1_nc:target

    // R_AARCH64_MOVW_PREL_G2
    movz x5, #:prel_g2:target

    // R_AARCH64_MOVW_PREL_G2_NC
    movk x6, #:prel_g2_nc:target

    // R_AARCH64_MOVW_PREL_G3
    movz x7, #:prel_g3:target
