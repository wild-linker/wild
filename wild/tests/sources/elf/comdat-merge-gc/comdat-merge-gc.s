//#AbstractConfig:base
//#Arch:x86_64
//#ReferenceLinkers:bfd,lld

//#Config:gc:base
//#LinkArgs:--gc-sections
//#NoSym:dead
//#LinkArgs:-no-pie

//#Config:roots:gc
# BFD retains companions when the group's merged strings are live. For now at least, we match LLD's
# behaviour of not doing this because it's slightly simpler.
//#ReferenceLinkers:lld
//#Object:roots.s
//#LinkerScript:keep.ld
//#LinkArgs:-u live_string
//#Contains:live string
//#Contains:retained string
//#Contains:kept string
//#NoSym:live_companion
//#NoSym:retained_companion
//#NoSym:kept_companion

//#Config:no-gc:base
//#LinkArgs:--no-gc-sections
//#ExpectError:missing
//#LinkArgs:-no-pie

.text
.globl _start
_start:
  mov $42, %edi
  mov $60, %eax
  syscall

.section .text.dead,"axG",@progbits,dead,comdat
.weak dead
dead:
  call missing
  ret
.section .rodata.dead,"aMSG",@progbits,1,dead,comdat
.asciz "unused string"

.section .note.GNU-stack,"",@progbits
