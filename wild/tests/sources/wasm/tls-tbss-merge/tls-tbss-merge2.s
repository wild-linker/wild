    .section .tdata.b,"T",@
    .globl tls_b
    .p2align 2
tls_b:
    .int32 222
    .size tls_b, 4

    .section .tbss.other,"T",@
    .globl tbss_other
    .p2align 2
tbss_other:
    .zero 16
    .size tbss_other, 16

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
