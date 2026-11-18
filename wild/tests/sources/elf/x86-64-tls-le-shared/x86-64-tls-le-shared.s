// Checks that a local-exec TLS access (R_X86_64_TPOFF32) is an error when linking a shared object,
// as for aarch64-tls-le-shared.
//#Arch:x86_64
//#LinkArgs:-shared
//#ReferenceLinkers:bfd,lld
//#ExpectError:R_X86_64_TPOFF32.*tvar.*shared

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
