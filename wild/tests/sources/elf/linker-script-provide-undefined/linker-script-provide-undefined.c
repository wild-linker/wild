//#Object:runtime.c
//#AugmentLinkerScript:provide.ld
//#LinkArgs:--undefined=provided
//#ExpectSym:provided address=0x1234
//#ExpectSym:provided_base address=0x1200
//#NoSym:unused_provided

#include "../common/runtime.h"

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
