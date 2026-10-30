//#AbstractConfig:base
//#Arch:x86_64
//#Mode:static
//#RunEnabled:false
//#ReferenceLinkers:bfd
//#Object:other.s

//#AbstractConfig:merged:base
//#ExpectSectionBytes:.note.gnu.property=0x040000002000000005000000474e5500010001c0040000000300000000000000020001c0040000000300000000000000

//#Config:gcc:merged

//#Config:clang:merged
//#Compiler:clang

//#Config:missing:base
//#Object:unmarked.s
//#NoSection:.note.gnu.property

.section .note.gnu.property,"a",@note
.balign 8
.long 4, 32, 5
.asciz "GNU"
.long 0xc0010001 # GNU_PROPERTY_X86_FEATURE_2_USED
.long 4, 1, 0
.long 0xc0010002 # GNU_PROPERTY_X86_ISA_1_USED
.long 4, 1, 0

.text
.globl _start
_start:
    ret

.section .note.GNU-stack,"",@progbits
