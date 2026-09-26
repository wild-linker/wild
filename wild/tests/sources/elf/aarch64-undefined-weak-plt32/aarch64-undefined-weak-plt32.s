// Check that an AArch64 PLT32 relocation against an undefined weak symbol
// resolves the symbol to the relocation place, so S - P + A = A.
//
//#Arch:aarch64
//#Compiler:clang
//#ReferenceLinkers:lld
//#RunEnabled:false
//#LinkArgs:--image-base=0x10000000 --no-gc-sections
//#ExpectSectionBytes:.data=0x00000000
//#DiffIgnore:file-header.entry

.weak target

.text
.global _start
_start:
    ret

.data
.global value
value:
    .word target@plt - .
