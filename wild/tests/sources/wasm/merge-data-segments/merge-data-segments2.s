    .section .rodata.other,"",@
    .globl ro_from_obj2
    .p2align 2
ro_from_obj2:
    .int32 0x778899aa
    .size ro_from_obj2, 4

    .section .data.arr,"",@
    .globl data_arr
    .p2align 3
data_arr:
    .int32 100
    .int32 200
    .size data_arr, 8

    .section mysection,"",@
    .globl custom_b
    .p2align 2
custom_b:
    .int32 22
    .size custom_b, 4

    .section .custom_section.target_features,"",@
    .int8 2
    .int8 43
    .int8 7
    .ascii "atomics"
    .int8 43
    .int8 11
    .ascii "bulk-memory"
