//#ReferenceLinkers:
//#ExpectError:relocation to a Wasm passive data segment is not supported yet

    .globl _start
_start:
    .functype _start () -> ()
    i32.const bytes
    drop
    end_function

    .section .data.bytes,"p",@
    .p2align 2
    .globl bytes
bytes:
    .int32 42
    .size bytes, 4
