# Verify that unknown GNU property types are ignored rather than causing errors.
# The object contains both an unknown property (0xc0000000) and a known one
# (0xc0000002 = IBT+SHSTK). Wild should ignore the unknown one and process
# the known one correctly.
//#Arch:x86_64
//#Mode:static
//#ReferenceLinkers:lld
//#RunEnabled:false

.section ".note.gnu.property", "a"
.align 4
.long 4
.long 32
.long 5
.asciz "GNU"

.long 0xc0000000
.long 4
.long 0
.long 0

.long 0xc0000002
.long 4
.long 3
.long 0

.text
.globl _start
_start:
  ret
