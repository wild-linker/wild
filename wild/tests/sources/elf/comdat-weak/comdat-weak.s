//#Arch:x86_64
//#Object:other.s
//#ReferenceLinkers:
//#ExpectErrorWild:reference to discarded COMDAT section `.text.missing`
//#LinkArgs:-no-pie

.text
.globl _start
_start:
  .weak missing
  mov $missing, %rax
  test %rax, %rax
  jne fail
  mov $42, %edi
  jmp exit
fail:
  mov $1, %edi
exit:
  mov $60, %eax
  syscall

.section .data.first,"awG",@progbits,signature,comdat
.globl first
first:
  .long 42
.section .note.GNU-stack,"",@progbits
