// A reference to a string-merge section via its section symbol with an addend outside the section
// must be rejected. GNU ld exits 0 on this input, so we don't compare against it.
//#Arch:x86_64
//#ReferenceLinkers:
//#LinkArgs:-no-pie
//#ExpectError:Merge-string offset 0xfffffffffffffffc is outside its input section of size 0x3

.section .text,"ax",@progbits
.globl _start
_start:
  ret

.section .rodata.str1.1,"aMS",@progbits,1
  .asciz "xy"

.section .data.retain,"awR",@progbits
  .long .rodata.str1.1 - . - 4

.section .note.GNU-stack,"",@progbits
