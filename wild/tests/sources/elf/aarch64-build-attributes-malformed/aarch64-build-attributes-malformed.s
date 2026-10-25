// Malformed AArch64 build attributes must cause the link to fail.
//
// The final two bytes extend beyond the declared subsection length.
//#Arch:aarch64
//#Compiler:clang
//#ReferenceLinkers:
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectErrorWild:Cannot parse .ARM.attributes section
//#ExpectErrorWild:unexpected end of data at offset 0x3f
//#LinkArgs:-no-pie

.text
.globl _start
_start:
    ret

.section .ARM.attributes,"",%0x70000003
.byte 0x41

// Valid PAuth subsection.
.long 0x00000019
.asciz "aeabi_pauthabi"
.byte 0x00, 0x00
.byte 0x01, 0x01, 0x02, 0x01

// Feature subsection with two extra bytes.
.long 0x00000023
.asciz "aeabi_feature_and_bits"
.byte 0x01, 0x00
.byte 0x00, 0x01, 0x01, 0x01, 0x02, 0x01
.byte 0x00, 0x00
