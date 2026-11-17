// Checks that a local-exec TLS access (R_AARCH64_TLSLE_ADD_TPREL_HI12/LO12_NC) is an error when
// linking a shared object. Local-exec is a fixed offset from the thread pointer into the
// executable's TLS block, and a shared object's TLS variables aren't in that block.
// GNU ld accepts this and produces a broken library, so there's no reference linker.
//#Arch:aarch64
//#LinkArgs:-shared
//#ReferenceLinkers:
//#ExpectErrorWild:R_AARCH64_TLSLE_ADD_TPREL_HI12 against symbol `tls_var` cannot be used when making a shared object

.section .tdata,"awT",%progbits
.globl tls_var
.type tls_var,%tls_object
tls_var:
  .word 5
.size tls_var, 4

.section .text,"ax",%progbits
.globl read_tls
.type read_tls,%function
read_tls:
  mrs x0, tpidr_el0
  add x0, x0, :tprel_hi12:tls_var
  add x0, x0, :tprel_lo12_nc:tls_var
  ldr w0, [x0]
  ret
.size read_tls, .-read_tls
