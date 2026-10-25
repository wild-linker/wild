//#Arch:x86_64
//#Archive:unused.s
//#Archive:used.s
//#NoSym:unused
//#NoSym:losing_only
//#LinkArgs:-no-pie

.text
.globl _start
_start:
  call helper
  mov foo(%rip), %edi
  mov $60, %eax
  syscall
.section .data.foo,"awG",@progbits,signature,comdat
.globl foo
foo:
  .long 42
.section .note.GNU-stack,"",@progbits
