// Checks local-dynamic TLS in an executable. The TLS index's offset must be 0, so that
// __tls_get_addr returns the start of the module's TLS block, and tvar's DTPREL must be its offset
// from there. main returns tvar's value.
//#AbstractConfig:default
//#Arch:aarch64
//#LinkerDriver:gcc
// lld doesn't support AArch64 local-dynamic TLS relocations.
//#ReferenceLinkers:bfd
//#DiffIgnore:section.data
//#DiffIgnore:section.rodata
// GNU ld relaxes the local-dynamic sequence to local-exec, which linker-diff can't match.
//#DiffIgnore:rel.reference_failed.R_AARCH64_TLSLD_ADR_PAGE21

//#Config:static:default
//#LinkArgs:-static

//#Config:no-pie:default
//#LinkArgs:-no-pie

.text
.globl main
.type main,%function
main:
  stp  x29, x30, [sp, #-16]!
  adrp x0, :tlsldm:tvar
  add  x0, x0, :tlsldm_lo12_nc:tvar
  bl   __tls_get_addr
  add  x0, x0, #:dtprel_hi12:tvar, lsl #12
  add  x0, x0, #:dtprel_lo12_nc:tvar
  ldr  x0, [x0]
  ldp  x29, x30, [sp], #16
  ret
.size main, .-main

.section .tdata,"awT",%progbits
.balign 8
other:
  .xword 7
tvar:
  .xword 42

.section .note.GNU-stack,"",%progbits
