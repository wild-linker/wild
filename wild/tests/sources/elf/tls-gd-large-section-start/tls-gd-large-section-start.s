//#Arch:x86_64
//#LinkArgs:-z noexecstack
// LLD 21 misidentifies the large-model sequence at section offset 0.
//#ReferenceLinkers:bfd
//#RunEnabled:false

.text
.globl _start
_start:
    // The TLSGD relocation is at offset 3, with no preceding padding.
    lea tlsvar@TLSGD(%rip), %rdi
    movabs $__tls_get_addr@PLTOFF, %rax
    add %rbx, %rax
    call *%rax
    ret

.section .tdata,"awT",@progbits
tlsvar:
    .quad 42

.section .note.GNU-stack,"",@progbits
