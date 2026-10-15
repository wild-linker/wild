//#Arch:x86_64
//#Object:other.s
//#ExpectError:foo
//#RunEnabled:false
//#LinkArgs:--no-gc-sections

.section .text.foo,"axG",@progbits,first_signature,comdat
.globl foo, _start
_start:
foo:
  ret
.section .note.GNU-stack,"",@progbits
