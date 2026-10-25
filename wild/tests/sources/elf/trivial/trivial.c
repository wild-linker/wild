//#Object:runtime.c
//#ExpectSym:_start section=".text"
//#ExpectSym:exit_syscall section=".text"
//#NoSym:__ehdr_start
//#NoSym:__executable_start
//#ReferenceLinkers:bfd,lld
//#TestUpdateInPlace:true
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
