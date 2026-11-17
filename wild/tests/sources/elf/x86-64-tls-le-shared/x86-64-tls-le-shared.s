// Checks that a local-exec TLS access (R_X86_64_TPOFF32) is an error when linking a shared object,
// as for aarch64-tls-le-shared.
// GNU ld accepts this and produces a broken library, so there's no reference linker.
//#Arch:x86_64
//#LinkArgs:-shared
//#ReferenceLinkers:
//#ExpectErrorWild:R_X86_64_TPOFF32 against symbol `tvar` cannot be used when making a shared object

.section .tdata,"awT",@progbits
.globl tvar
.type tvar,@object
tvar:
  .long 5
.size tvar, 4

.text
.globl read_tls
.type read_tls,@function
read_tls:
  movl %fs:tvar@tpoff, %eax
  ret
.size read_tls, .-read_tls
