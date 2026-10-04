//#Arch:ppc64le
//#Compiler:clang
//#Mode:dynamic
//#Shared:callee.c
//#SoSingleLinker:lld
//#ReferenceLinkers:bfd,lld
//#LinkArgs:-z now
//#ExpectError:TOC|toc

.abiversion 2
.text
.globl _start
.type _start, @function
_start:
  bl callee
  li 3, 42
  blr
.size _start, .-_start

.section .note.GNU-stack,"",@progbits
