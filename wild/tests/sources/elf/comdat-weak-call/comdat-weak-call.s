//#Arch:aarch64
//#Object:other.s
//#ReferenceLinkers:
//#ExpectErrorWild:reference to discarded COMDAT section `.text.missing`

.section .data.first,"awG",@progbits,signature,comdat
.long 42
.section .note.GNU-stack,"",@progbits
