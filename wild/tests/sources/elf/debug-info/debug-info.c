//#Config:default
//#CompArgs:-g
//#Object:runtime.c
//#ExpectSym:test_func line=11
//#ExpectSym:_start line=13
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

int test_func(void) { return 10; }

void _start(void) {
  runtime_init();

  if (test_func() != 10) {
    exit_syscall(11);
  }

  exit_syscall(42);
}
