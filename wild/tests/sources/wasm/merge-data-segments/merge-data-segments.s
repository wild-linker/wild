//#AbstractConfig:base
//#Object:merge-data-segments2.s

//#Config:default:base
//#ExpectDataSegmentCount: 3
//#ExpectDataSegmentName: 0=.rodata
//#ExpectDataSegmentName: 1=.data
//#ExpectDataSegmentName: 2=mysection

//#Config:explicit-merge:base
//#LinkArgs: --no-merge-data-segments --merge-data-segments
//#ExpectDataSegmentCount: 3
//#ExpectDataSegmentName: 0=.rodata
//#ExpectDataSegmentName: 1=.data
//#ExpectDataSegmentName: 2=mysection

//#Config:no-merge:base
//#LinkArgs: --no-merge-data-segments
//#ExpectDataSegmentCount: 6

//#Config:shared-memory:base
//#LinkArgs: --shared-memory --max-memory=131072
//#ExpectSharedMemory: true
//#ExpectSection: DataCount
//#ExpectSection: Start
//#ExpectDataSegmentCount: 3
//#ExpectDataSegmentName: 0=.rodata
//#ExpectDataSegmentName: 1=.data
//#ExpectDataSegmentName: 2=mysection

    .globl _start
_start:
    .functype _start () -> ()

    # Check alignment of ro_aligned16 (16-byte aligned).
    i32.const ro_aligned16
    i32.const 15
    i32.and
    if
      unreachable
    end_if

    # Check ro_byte == 0x5A and ro_aligned16 == 0x11223344.
    i32.const ro_byte
    i32.load8_u 0
    i32.const 0x5a
    i32.ne
    if
      unreachable
    end_if

    i32.const ro_aligned16
    i32.load 0
    i32.const 0x11223344
    i32.ne
    if
      unreachable
    end_if

    # Check ro_from_obj2 == 0x778899AA.
    i32.const ro_from_obj2
    i32.load 0
    i32.const 0x778899aa
    i32.ne
    if
      unreachable
    end_if

    # Check data_ptr points to data_arr + 4 (value 200).
    i32.const data_ptr
    i32.load 0
    i32.load 0
    i32.const 200
    i32.ne
    if
      unreachable
    end_if

    # Check custom section values merged from both objects.
    i32.const custom_a
    i32.load 0
    i32.const 11
    i32.ne
    if
      unreachable
    end_if

    i32.const custom_b
    i32.load 0
    i32.const 22
    i32.ne
    if
      unreachable
    end_if

    end_function

    .section .rodata.b,"",@
    .globl ro_byte
    .p2align 0
ro_byte:
    .int8 0x5a
    .size ro_byte, 1

    .section .rodata.a16,"",@
    .globl ro_aligned16
    .p2align 4
ro_aligned16:
    .int32 0x11223344
    .size ro_aligned16, 4

    .section .data.ptr,"",@
    .globl data_ptr
    .p2align 2
data_ptr:
    .int32 data_arr+4
    .size data_ptr, 4

    .section mysection,"",@
    .globl custom_a
    .p2align 2
custom_a:
    .int32 11
    .size custom_a, 4

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
