//#ReferenceLinkers:
//#ExpectSection:Data
//#ExpectSection:DataCount

    .globl _start
_start:
    .functype _start () -> ()
    i32.const tls1@TLSREL
    drop
    i32.const 16
    i32.const 0
    i32.const 4
    memory.init 1, 0
    i32.const 16
    i32.load 0
    i32.const 42
    i32.ne
    if
      unreachable
    end_if
    end_function

    .section .tdata.tls1,"T",@
    .p2align 2
    .globl tls1
tls1:
    .int32 1
    .size tls1, 4

    .section .data.bytes,"p",@
    .p2align 2
    .globl bytes
bytes:
    .int32 42
    .size bytes, 4

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
