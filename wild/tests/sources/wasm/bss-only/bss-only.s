//#Config:default
//#NoSection: Data
//#ExpectDataSegmentCount: 0

//#Config:no-merge
//#LinkArgs: --no-merge-data-segments
//#NoSection: Data
//#ExpectDataSegmentCount: 0

    .globl _start
_start:
    .functype _start () -> ()

    # Verify both BSS symbols are zeroed and placed before __data_end.
    i32.const bss_a
    i32.load 0
    if
      unreachable
    end_if

    i32.const bss_b
    i32.load 0
    if
      unreachable
    end_if

    i32.const bss_b
    i32.const 4096
    i32.add
    i32.const __data_end
    i32.gt_u
    if
      unreachable
    end_if

    end_function

    .section .bss.a,"",@
    .globl bss_a
    .p2align 2
bss_a:
    .zero 16
    .size bss_a, 16

    .section .bss.b,"",@
    .globl bss_b
    .p2align 4
bss_b:
    .zero 4096
    .size bss_b, 4096
