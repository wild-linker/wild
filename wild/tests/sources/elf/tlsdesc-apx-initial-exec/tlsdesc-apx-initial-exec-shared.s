.section .tdata,"awT",@progbits
.globl tlsvar
.type tlsvar, @tls_object
tlsvar:
    .quad 42

.section .note.GNU-stack,"",@progbits
