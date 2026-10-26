//#Arch:aarch64
//#Mode:dynamic
//#LinkArgs:-shared --no-gc-sections
//#ExpectError:R_AARCH64_ABS32
//#ExpectErrorWild:R_AARCH64_ABS32.*cannot use a runtime address relocation.*full 8-byte address field
.globl hidden
.hidden hidden
hidden:
.data
.long hidden
