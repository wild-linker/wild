//#Object:runtime.c
//#Object:sections.s
//#ExpectSym:foo alignment=0x10000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
