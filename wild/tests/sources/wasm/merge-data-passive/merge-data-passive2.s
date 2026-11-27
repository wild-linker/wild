    .globl check_merged
check_merged:
    .functype check_merged () -> ()
    i32.const ro1
    i32.load 0
    i32.const 10
    i32.ne
    if
      unreachable
    end_if
    i32.const ro2
    i32.load 0
    i32.const 20
    i32.ne
    if
      unreachable
    end_if
    i32.const d1
    i32.load 0
    i32.const 30
    i32.ne
    if
      unreachable
    end_if
    i32.const d2
    i32.load 0
    i32.const 40
    i32.ne
    if
      unreachable
    end_if
    end_function

    .section .rodata.r1,"",@
    .p2align 2
ro1:
    .int32 10
    .size ro1, 4

    .section .rodata.r2,"",@
    .p2align 2
ro2:
    .int32 20
    .size ro2, 4

    .section .data.d1,"",@
    .p2align 2
d1:
    .int32 30
    .size d1, 4

    .section .data.d2,"",@
    .p2align 2
d2:
    .int32 40
    .size d2, 4
