// Defines the same hidden symbols as dup-two.s, in COMDAT groups like GCC and clang.
.section .text.address_one,"ax",@progbits
.globl address_one
address_one:
  lea hidden_weak(%rip), %rax
  lea hidden_unique(%rip), %rax
  ret

.section .data.hidden_weak,"awG",@progbits,hidden_weak,comdat
.weak hidden_weak
.hidden hidden_weak
.type hidden_weak, @object
.size hidden_weak, 4
hidden_weak:
  .long 10

.section .data.hidden_unique,"awG",@progbits,hidden_unique,comdat
.globl hidden_unique
.hidden hidden_unique
.type hidden_unique, @gnu_unique_object
.size hidden_unique, 4
hidden_unique:
  .long 10
.section .note.GNU-stack,"",@progbits
