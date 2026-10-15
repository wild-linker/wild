//#AbstractConfig:base
//#Arch:x86_64
//#ReferenceLinkers:bfd,lld

//#Config:gc:base
//#LinkArgs:--gc-sections
//#NoSym:dead
//#NoSym:dead_debug
//#NoSection:.debug_dead
//#ExpectSym:live_companion
//#ExpectSectionBytes:.debug_live=0x78563412
//#ExpectSectionBytes:.debug_only=0x11111111
//#ExpectSectionBytes:.debug_only_companion=0x22222222
//#ExpectSectionBytes:.debug_ungrouped=0x33333333

//#Config:debug-reference:gc
# BFD retains the group when another debug section refers to its debug member.
//#ReferenceLinkers:lld
//#CompArgs:-Wa,--defsym,DEBUG_REFERENCE=1
//#ExpectSectionBytes:.debug_reference=0x0000000000000000

//#Config:no-gc:base
//#LinkArgs:--no-gc-sections
//#ExpectError:missing

.text
.globl _start
_start:
  mov live(%rip), %edi
  mov $60, %eax
  syscall

.section .text.dead,"axG",@progbits,dead,comdat
.weak dead
dead:
  call missing
  ret
.section .debug_dead,"G",@progbits,dead,comdat
dead_debug:
.long 0x87654321

.section .data.live,"awG",@progbits,live,comdat
.globl live
live:
  .long 42
.section .data.live_companion,"awG",@progbits,live,comdat
.globl live_companion
live_companion:
  .long 17
.section .debug_live,"G",@progbits,live,comdat
.long 0x12345678

.section .debug_only,"G",@progbits,debug_only,comdat
.long 0x11111111
.section .debug_only_companion,"G",@progbits,debug_only,comdat
.long 0x22222222
.section .debug_ungrouped,"",@progbits
.long 0x33333333

.ifdef DEBUG_REFERENCE
.section .debug_reference,"",@progbits
.quad dead_debug
.endif

.section .note.GNU-stack,"",@progbits
