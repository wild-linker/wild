//#Arch:ppc64le
//#Compiler:clang
//#Mode:dynamic
//#Shared:lib.c
//#LinkArgs:-no-pie -z now
//#ReferenceLinkers:bfd
//#ExpectError:ADDR32

.abiversion 2
.text
.globl resolver
.type resolver, @function
resolver:
  li 3, 0
  blr
.globl selected
.type selected, @gnu_indirect_function
.set selected, resolver

.globl _start
.type _start, @function
_start:
  blr
  .long selected

.section .note.GNU-stack,"",@progbits
