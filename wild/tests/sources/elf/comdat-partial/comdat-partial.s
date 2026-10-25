//#Arch:x86_64
//#Relocatable:group.s,group.s
//#Object:group.s
//#LinkArgs:-no-pie

.text
.globl _start
_start:
  mov ptr(%rip), %rax
  mov (%rax), %edi
  add bss(%rip), %edi
  mov $60, %eax
  syscall
.section .note.GNU-stack,"",@progbits
