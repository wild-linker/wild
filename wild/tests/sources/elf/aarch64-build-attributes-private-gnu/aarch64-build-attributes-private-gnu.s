// An optional private subsection does not conflict with a GNU BTI property.
//
//#Arch:aarch64
//#Compiler:clang
//#ReferenceLinkers:
//#LinkArgs:-r
//#RunEnabled:false
//#DiffEnabled:false
//#NoSection:.ARM.attributes
//#ExpectSectionBytes:.note.gnu.property=0x040000001000000005000000474e5500000000c0040000000100000000000000

.text
.globl private_input
private_input:
    ret

.aeabi_subsection anon_dummy, optional, uleb128
.aeabi_attribute 1, 1

.section .note.gnu.property,"a",@note
.p2align 3
.long 4
.long 16
.long 5
.asciz "GNU"
.long 0xc0000000 // GNU_PROPERTY_AARCH64_FEATURE_1_AND
.long 4
.long 1 // GNU_PROPERTY_AARCH64_FEATURE_1_BTI
.long 0
