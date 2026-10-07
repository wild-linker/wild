//#LinkArgs: --shared-memory --max-memory=131072
//#ReferenceLinkers:
//#ExpectError: relocation to a TLS symbol in a shared-memory TLS template is not supported yet

    .globaltype __tls_base, i32

    .globl _start
_start:
    .functype _start () -> ()
    global.get __tls_base
    i32.const tls1@TLSREL
    i32.add
    drop
    end_function

    .section .tdata.tls1,"T",@
    .p2align 2
    .globl tls1
tls1:
    .int32 tls2
    .size tls1, 4

    .section .tdata.tls2,"T",@
    .p2align 2
    .globl tls2
tls2:
    .int32 1
    .size tls2, 4

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
