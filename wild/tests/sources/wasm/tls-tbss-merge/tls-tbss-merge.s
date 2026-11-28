//#AbstractConfig:base
//#Object:tls-tbss-merge2.s

//#Config:default:base
//#ExpectDataSegmentCount: 2

//#Config:shared-memory:base
//#LinkArgs: --shared-memory --max-memory=131072
//#ReferenceLinkers:
//#ExpectSharedMemory: true
//#ExpectSection: DataCount
//#ExpectSection: Start
//#ExpectDataSegmentCount: 2
//#ExpectDataSegmentName: 0=.data
//#ExpectDataSegmentName: 1=.tdata

    .globaltype __tls_base, i32
    .functype __wasm_init_tls (i32) -> ()

    .globl _start
_start:
    .functype _start () -> ()

    # Ordinary data check.
    i32.const ord_val
    i32.load 0
    i32.const 99
    i32.ne
    if
      unreachable
    end_if

    # Verify TLS values in the main thread (or non-shared TLS block).
    global.get __tls_base
    i32.const tls_a@TLSREL
    i32.add
    i32.load 0
    i32.const 111
    i32.ne
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tls_b@TLSREL
    i32.add
    i32.load 0
    i32.const 222
    i32.ne
    if
      unreachable
    end_if

    # .tbss is 16-byte aligned and zeroed.
    global.get __tls_base
    i32.const tbss_arr@TLSREL
    i32.add
    i32.const 15
    i32.and
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tbss_arr@TLSREL
    i32.add
    i32.load 0
    if
      unreachable
    end_if

    global.get __tls_base
    i32.const tbss_other@TLSREL
    i32.add
    i32.load 0
    if
      unreachable
    end_if

    end_function

    .section .data.ord,"",@
    .globl ord_val
    .p2align 2
ord_val:
    .int32 99
    .size ord_val, 4

    .section .bss.ord_bss,"",@
    .globl ord_bss
    .p2align 2
ord_bss:
    .zero 64
    .size ord_bss, 64

    .section .tdata.a,"T",@
    .globl tls_a
    .p2align 2
tls_a:
    .int32 111
    .size tls_a, 4

    .section .tbss.zero,"T",@
    .globl tbss_arr
    .p2align 4
tbss_arr:
    .zero 32
    .size tbss_arr, 32

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
