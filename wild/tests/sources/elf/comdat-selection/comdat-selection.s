//#AbstractConfig:common
//#Arch:x86_64
//#Object:other.s
//#ExpectSym:companion

//#AbstractConfig:base:common
//#NoSym:losing_only
//#Config:gc:base
//#Config:dead-reference:base
//#Object:dead.s
//#NoSection:.data.unused
//#Config:single-thread:base
//#WildExtraLinkArgs:--threads=1
//#Env:WILD_FILES_PER_GROUP=1
//#Config:multiple-threads:base
//#WildExtraLinkArgs:--threads=4
//#Env:WILD_FILES_PER_GROUP=2
//#Config:no-gc:base
//#LinkArgs:--no-gc-sections

//#Config:partial:common
//#Object:frame.s
//#LinkArgs:-r
//#RunEnabled:false
//#DiffEnabled:false
# BFD omits unreferenced discarded globals; LLD emits them as undefined symbols.
//#ReferenceLinkers:lld
//#ExpectSym:losing_only binding=global
//#NoSection:.data.losing_only
//#NoSection:.text.losing

//#Config:shared:base
//#LinkArgs:-shared -Bsymbolic
//#Mode:dynamic
//#RunEnabled:false
//#DiffEnabled:false

.text
.globl _start
_start:
  mov foo(%rip), %edi
  mov $60, %eax
  syscall

.section .data.foo,"awG",@progbits,signature,comdat
.globl foo
foo:
  .long 42

.section .data.companion,"awG",@progbits,signature,comdat
.globl companion
companion:
  .long 17

.section .note.GNU-stack,"",@progbits
