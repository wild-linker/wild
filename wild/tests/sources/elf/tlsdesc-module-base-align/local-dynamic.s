// This is done in assembly to avoid the compiler optimising away the reference to
// _TLS_MODULE_BASE_.
.text
.globl local_dynamic_address
.type local_dynamic_address, @function
local_dynamic_address:
    subq $8, %rsp
    leaq _TLS_MODULE_BASE_@TLSDESC(%rip), %rax
    call *_TLS_MODULE_BASE_@TLSCALL(%rax)
    addq $value@dtpoff, %rax
    addq %fs:0, %rax
    addq $8, %rsp
    ret
.size local_dynamic_address, .-local_dynamic_address

.section .note.GNU-stack,"",@progbits
