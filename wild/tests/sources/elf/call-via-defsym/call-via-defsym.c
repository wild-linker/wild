//#Config:default
//#Object:runtime.c
//#Object:call-via-defsym-1.c
//#LinkArgs:-znow --defsym=foo=bar
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

int foo();
int __attribute__((weak)) bar(void) { return 8; }

void _start(void) {
  runtime_init();
  exit_syscall(foo());
}
