// Checks that hidden symbols stay global in a partial link's output, so that a later link can still
// resolve references to them. Covers hidden symbols defined in two files, weak and GNU-unique as
// emitted for inline variables, and a default-visibility definition that a hidden reference points
// to.
//#Arch:x86_64
//#Relocatable:dup-one.s,dup-two.s,hidden-ref.s,default-def.s
//#LinkArgs:-no-pie

//#Config:shared:default
//#Mode:dynamic
//#PostLinkArgs:-shared
//#RunEnabled:false
//#NoDynSym:referenced_hidden

.text
.globl _start
_start:
  mov hidden_weak(%rip), %edi
  add hidden_unique(%rip), %edi
  add referenced_hidden(%rip), %edi
  mov $60, %eax
  syscall
.section .note.GNU-stack,"",@progbits
