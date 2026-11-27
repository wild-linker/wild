//#Arch:ppc64le
//#Compiler:clang
//#ReferenceLinkers:lld
//#LinkArgs:-no-pie
//#RunEnabled:false
//#ExpectError:ABI version 1

.abiversion 1
.text
.globl _start
.type _start, @function
_start:
  blr

.section .note.GNU-stack,"",@progbits
