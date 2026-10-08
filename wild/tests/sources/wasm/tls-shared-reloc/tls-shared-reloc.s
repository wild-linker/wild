//#LinkArgs: --shared-memory --max-memory=131072
//#ReferenceLinkers:
//#ExpectError: relocation to a shared-memory TLS symbol is not supported yet

    .globl _start
_start:
    .functype _start () -> ()
    i32.const tls1
    drop
    end_function

    .section .tdata.tls1,"T",@
    .p2align 2
    .globl tls1
tls1:
    .int32 1
    .size tls1, 4

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
