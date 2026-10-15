//#Arch:x86_64
//#Object:other.s
//#ExpectSym:companion

.text
.globl _start
_start:
  mov foo(%rip), %edi
  add bar(%rip), %edi
  mov $60, %eax
  syscall
.section .data.foo,"awG",@progbits,signature
.globl foo
foo:
  .long 20
.section .data.companion,"awG",@progbits,signature
.globl companion
companion:
  .long 17
.section .note.GNU-stack,"",@progbits
