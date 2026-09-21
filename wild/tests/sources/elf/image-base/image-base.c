// Verify that --image-base sets the load address of the output binary.
// Also verifies that an error is emitted when the address is not page-aligned.
//#AbstractConfig:default
//#Mode:static
//#RunEnabled:false
//#Object:runtime.c

//#Config:aligned:default
//#ReferenceLinkers:bfd,lld
//#LinkArgs:--image-base=0x10000000 --undefined=__executable_start
//#ExpectSym:__executable_start address=0x10000000

//#Config:unaligned:default
//#ReferenceLinkers:
//#LinkArgs:--image-base=0x10000001
//#ExpectError:--image-base: address isn't multiple of page size: 0x10000001

#include "../common/runtime.h"
void _start(void) { exit_syscall(42); }
