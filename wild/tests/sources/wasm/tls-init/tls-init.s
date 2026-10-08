// The runtime, here `_start`, calls `__wasm_init_tls` before any TLS access.
//#LinkArgs: --shared-memory --max-memory=131072 --export=__tls_base
//#ReferenceLinkers:
//#DiffEnabled:false
//#ExpectSym: __tls_base address=0
//#ExpectSym: __tls_size address=12
//#ExpectSym: __tls_align address=4
//#ExpectSym: __wasm_init_tls
//#ExpectSection: DataCount
//#ExpectSharedMemory: true

    .globaltype __tls_base, i32
    .globaltype __tls_size, i32, immutable
    .globaltype __tls_align, i32, immutable
    .functype __wasm_init_tls (i32) -> ()

    .globl _start
_start:
    .functype _start () -> ()

    global.get __tls_size
    i32.const 12
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_align
    i32.const 4
    i32.ne
    if
      unreachable
    end_if

    i32.const tls_space
    call __wasm_init_tls

    global.get __tls_base
    i32.const tls_space
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tls1@TLSREL
    i32.add
    i32.load 0
    i32.const 42
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tls2@TLSREL
    i32.add
    i32.load 0
    i32.const 43
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tls_bss@TLSREL
    i32.add
    i32.load 0
    i32.const 0
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tls1@TLSREL
    i32.add
    i32.const 7
    i32.store 0

    global.get __tls_base
    i32.const tls1@TLSREL
    i32.add
    i32.load 0
    i32.const 7
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

    .section .tdata.tls2,"T",@
    .p2align 2
    .globl tls2
tls2:
    .int32 43
    .size tls2, 4

    .section .tbss.tls_bss,"T",@
    .p2align 2
    .globl tls_bss
tls_bss:
    .int32 0
    .size tls_bss, 4

    .section .data.space,"",@
    .p2align 2
    .globl tls_space
tls_space:
    .zero 32
    .size tls_space, 32

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
