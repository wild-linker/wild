/*
//#AbstractConfig:default
//#Arch:x86_64
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false

//#Config:none:default
//#CompArgs:-Wa,--defsym,TEST_NONE=1
//#ExpectSectionBytes:.debug_test=0x8877665544332211
//#LinkArgs:-no-pie

//#Config:absolute:default
// LLD writes zeros for symbol-free absolute debug relocations.
//#ReferenceLinkers:bfd
//#CompArgs:-Wa,--defsym,TEST_NONE=0
//#ExpectSectionBytes:.debug_test=0x887766554433221178563412feffffff
//#LinkArgs:-no-pie
*/

.text
.globl _start
_start:
    ret

.section .debug_test,"",@progbits
.if TEST_NONE
    .reloc ., R_X86_64_NONE, 0
    .quad 0x1122334455667788
.else
    .reloc ., R_X86_64_64, 0x1122334455667788
    .quad 0
    .reloc ., R_X86_64_32, 0x12345678
    .long 0
    .reloc ., R_X86_64_32S, -2
    .long 0
.endif
