// Test that R_X86_64_32 to a DSO symbol errors in PIE/shared output.
// Both GNU ld and lld error with "recompile with -fPIC".
//#Arch:x86_64
//#Mode:dynamic
//#Shared:pie-invalid-reloc-dso.s
//#LinkArgs:-pie --no-gc-sections
//#ExpectError:R_X86_64_32
//#ExpectErrorWild:R_X86_64_32.*cannot use a runtime address relocation.*full 8-byte address field
.globl _start
_start:
  ret
.data
.long dso_sym
