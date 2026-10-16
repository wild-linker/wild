//#LinkArgs: --shared-memory --max-memory=131072
//#ReferenceLinkers:
//#ExpectSym: __wasm_init_tls
//#ExpectSharedMemory: true

    .globaltype __tls_base, i32
    .functype __wasm_init_tls (i32) -> ()

    .globl _start
_start:
    .functype _start () -> ()

    i32.const tls_space_a
    call __wasm_init_tls

    # *tls1 == &tls2 in this thread.
    global.get __tls_base
    i32.const tls1@TLSREL
    i32.add
    i32.load 0
    global.get __tls_base
    i32.const tls2@TLSREL
    i32.add
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tls1@TLSREL
    i32.add
    i32.load 0
    i32.load 0
    i32.const 42
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tls1_add@TLSREL
    i32.add
    i32.load 0
    global.get __tls_base
    i32.const tls2@TLSREL
    i32.add
    i32.const 4
    i32.add
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tls1_add@TLSREL
    i32.add
    i32.load 0
    i32.load 0
    i32.const 7
    i32.ne
    if
      unreachable
    end_if

    # Non-TLS target stays an absolute address.
    global.get __tls_base
    i32.const tls_ord@TLSREL
    i32.add
    i32.load 0
    i32.const ordinary
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tls_neg@TLSREL
    i32.add
    i32.load 0
    global.get __tls_base
    i32.const -4
    i32.add
    i32.ne
    if
      unreachable
    end_if

    i32.const tls_space_b
    call __wasm_init_tls

    global.get __tls_base
    i32.const tls1@TLSREL
    i32.add
    i32.load 0
    global.get __tls_base
    i32.const tls2@TLSREL
    i32.add
    i32.ne
    if
      unreachable
    end_if

    # The first thread's copy still points into its own block.
    i32.const tls_space_a
    i32.const tls1@TLSREL
    i32.add
    i32.load 0
    i32.const tls_space_a
    i32.const tls2@TLSREL
    i32.add
    i32.ne
    if
      unreachable
    end_if
    end_function

    .section .tdata.tls1,"T",@
    .p2align 2
    .globl tls1
tls1:
    .int32 tls2
    .size tls1, 4

    .globl tls_ord
tls_ord:
    .int32 ordinary
    .size tls_ord, 4

    .globl tls1_add
tls1_add:
    .int32 tls2+4
    .size tls1_add, 4

    .globl tls_neg
tls_neg:
    .int32 tls1-4
    .size tls_neg, 4

    .section .tdata.tls2,"T",@
    .p2align 2
    .globl tls2
tls2:
    .int32 42
    .int32 7
    .size tls2, 8

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
    .zero 32
    .size tls_space_a, 32

    .section .data.space_b,"",@
    .p2align 2
    .globl tls_space_b
tls_space_b:
    .zero 32
    .size tls_space_b, 32

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
