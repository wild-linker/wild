// `sym@GOT@TLS` is a mutable global. `__wasm_init_tls` sets it to this thread's
// `__tls_base + offset`. A non-TLS GOT keeps the absolute address.
//#LinkArgs: --shared-memory --max-memory=131072
//#ReferenceLinkers:
//#DiffEnabled:false
//#ExpectSym: __wasm_init_tls
//#ExpectSharedMemory: true

    .globaltype __tls_base, i32
    .functype __wasm_init_tls (i32) -> ()

    .globl _start
_start:
    .functype _start () -> ()

    i32.const tls_space_a
    call __wasm_init_tls

    global.get tls1@GOT@TLS
    global.get __tls_base
    i32.const tls1@TLSREL
    i32.add
    i32.ne
    if
      unreachable
    end_if

    global.get tls1@GOT@TLS
    i32.load 0
    i32.const 42
    i32.ne
    if
      unreachable
    end_if

    global.get tls2@GOT@TLS
    global.get __tls_base
    i32.const tls2@TLSREL
    i32.add
    i32.ne
    if
      unreachable
    end_if

    global.get tls2@GOT@TLS
    i32.load 0
    i32.const 7
    i32.ne
    if
      unreachable
    end_if

    global.get ordinary@GOT
    i32.const ordinary
    i32.ne
    if
      unreachable
    end_if

    i32.const tls_space_b
    call __wasm_init_tls

    global.get tls2@GOT@TLS
    global.get __tls_base
    i32.const tls2@TLSREL
    i32.add
    i32.ne
    if
      unreachable
    end_if

    global.get ordinary@GOT
    i32.const ordinary
    i32.ne
    if
      unreachable
    end_if
    end_function

    .section .tdata.tls1,"T",@
    .p2align 2
    .globl tls1
tls1:
    .int32 42
    .size tls1, 4

    .globl tls2
tls2:
    .int32 7
    .size tls2, 4

    .section .data.ordinary,"",@
    .p2align 2
    .globl ordinary
ordinary:
    .int32 99
    .size ordinary, 4

    .section .data.space_a,"",@
    .p2align 2
    .globl tls_space_a
tls_space_a:
    .zero 16
    .size tls_space_a, 16

    .section .data.space_b,"",@
    .p2align 2
    .globl tls_space_b
tls_space_b:
    .zero 16
    .size tls_space_b, 16

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
