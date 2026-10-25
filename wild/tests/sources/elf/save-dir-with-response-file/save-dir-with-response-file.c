//#Object:runtime.c
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
