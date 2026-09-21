# Verify that IBT+SHSTK properties are preserved in the output binary
# when the input object has them set.
# GNU_PROPERTY_X86_FEATURE_1_AND (0xc0000002) with value 3 (IBT|SHSTK).
//#Arch:x86_64
//#Mode:static
//#ReferenceLinkers:lld
//#RunEnabled:false
//#ExpectSectionBytes:.note.gnu.property=0x020000c00400000003000000 16..28

.section ".note.gnu.property", "a"
.align 4
.long 4
.long 16
.long 5
.asciz "GNU"

.long 0xc0000002
.long 4
.long 3
.long 0

.text
.globl _start
_start:
  ret
