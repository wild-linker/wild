//#Arch:x86_64
//#LinkArgs:-no-pie
//#LinkerScript:linker-script-empty-section-address.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectSym:_start address=0x400000

.text
.globl _start
_start:
  ret
