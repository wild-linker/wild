//#AbstractConfig:base
//#Arch:x86_64
# LLD 22 retains merge flags when entry sizes differ.
//#ReferenceLinkers:bfd
//#LinkArgs:-static --no-gc-sections
//#ExpectSection:.rodata entsize=0,flags=A
//#ExpectSection:uniform_strings entsize=1,flags=AMS
//#ExpectSection:uniform_constants entsize=8,flags=AM
//#ExpectSection:mixed_sizes entsize=0,flags=A

//#Config:single-group:base
//#Env:WILD_FILES_PER_GROUP=256
//#Object:first.s
//#Object:second.s
//#Object:third.s

//#Config:separate-groups:single-group
//#Env:WILD_FILES_PER_GROUP=1

//#Config:reverse:base
//#Env:WILD_FILES_PER_GROUP=1
//#Object:second.s
//#Object:first.s
//#Object:third.s

//#Config:script:separate-groups
//#LinkerScript:layout.ld

.text
.globl _start
_start:
  mov $42, %edi
  mov $60, %eax
  syscall

.section .note.GNU-stack,"",@progbits
