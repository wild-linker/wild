.section .note.gnu.property,"a",@note
.balign 8
.long 4, 32, 5
.asciz "GNU"
.long 0xc0010001 # GNU_PROPERTY_X86_FEATURE_2_USED
.long 4, 2, 0
.long 0xc0010002 # GNU_PROPERTY_X86_ISA_1_USED
.long 4, 2, 0

.section .note.GNU-stack,"",@progbits
