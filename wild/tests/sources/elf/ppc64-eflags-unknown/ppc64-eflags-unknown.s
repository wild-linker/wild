//#Arch:ppc64le
//#Compiler:clang
//#ReferenceLinkers:lld
//#LinkArgs:-no-pie
//#RunEnabled:false
//#ExpectError:[Uu]nrecognized.*e_flags

.abiversion 3
.text
.globl _start
.type _start, @function
_start:
  blr

.section .note.GNU-stack,"",@progbits
