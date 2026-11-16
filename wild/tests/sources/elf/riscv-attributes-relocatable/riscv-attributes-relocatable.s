//#Arch:riscv64
//#Compiler:clang
//#LinkArgs:-r
//#RunEnabled:false
//#ExpectSym:answer section=.text

.attribute arch, "rv64imafdc"

.text
.globl answer
answer:
    ret

// Mapping symbols in attributes sections must not require ordinary section addresses.
.section .riscv.attributes
$d:
