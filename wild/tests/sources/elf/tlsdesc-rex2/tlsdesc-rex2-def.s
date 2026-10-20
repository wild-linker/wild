.section .tdata,"awT",@progbits
.p2align 3
.globl tlsvar
.type tlsvar,@tls_object
tlsvar:
    .quad 42
.size tlsvar, .-tlsvar

.section .note.GNU-stack,"",@progbits
