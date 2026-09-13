//#DiffEnabled:false
//#DoesNotContain: env

    .globaltype __tls_base, i32

    .globl    _start
_start:
    .functype    _start () -> ()

    # tls1 is at TLS offset 0 and holds 42.
    global.get    __tls_base
    i32.const    tls1@TLSREL
    i32.add
    i32.load    0
    i32.const    42
    i32.ne
    if
      unreachable
    end_if

    # tls2 is at a non-zero TLS offset and holds 43.
    global.get    __tls_base
    i32.const    tls2@TLSREL
    i32.add
    i32.load    0
    i32.const    43
    i32.ne
    if
      unreachable
    end_if
    end_function

    .section    .tdata.tls1,"T",@
    .p2align    2
    .globl    tls1
tls1:
    .int32    42
    .size    tls1, 4

    .section    .tdata.tls2,"T",@
    .p2align    2
    .globl    tls2
tls2:
    .int32    43
    .size    tls2, 4

    .section    .custom_section.target_features,"",@
    .int8    2
    .int8    43
    .int8    7
    .ascii    "atomics"
    .int8    43
    .int8    11
    .ascii    "bulk-memory"
