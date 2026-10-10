// Verify that conflicting AArch64 build attributes and GNU properties are
// rejected during a normal executable link, not just a relocatable link.
//
//#Arch:aarch64
//#Compiler:clang
//#ReferenceLinkers:
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectErrorWild:GNU properties and build attributes have conflicting AArch64 PAuth data

.text
.globl _start
_start:
    ret

.aeabi_subsection aeabi_pauthabi, required, uleb128
.aeabi_attribute Tag_PAuth_Platform, 5
.aeabi_attribute Tag_PAuth_Schema, 5

.aeabi_subsection aeabi_feature_and_bits, optional, uleb128
.aeabi_attribute Tag_Feature_BTI, 1
.aeabi_attribute Tag_Feature_PAC, 1
.aeabi_attribute Tag_Feature_GCS, 1

.section ".note.gnu.property", "a", @note
.p2align 3
.long 4
.long property_end - property_begin
.long 5
.asciz "GNU"
.p2align 3

property_begin:
.long 0xc0000000
.long 4
.long 7
.long 0

.long 0xc0000001
.long 16
.quad 0x12345678
.quad 0x87654321
property_end:
