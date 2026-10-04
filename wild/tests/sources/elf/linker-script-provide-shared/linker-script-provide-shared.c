//#Object:runtime.c
//#Mode:dynamic
//#Shared:shared.c
//#CompSoArgs:-fPIC
//#AugmentLinkerScript:provide.ld
//#LinkArgs:-no-pie
//#ExpectDynSym:provided address=42
//#NoSym:unused_provided
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT

#include "../common/runtime.h"

unsigned long provided_value(void);

void _start(void) {
  runtime_init();
  if (provided_value() != 42) {
    exit_syscall(1);
  }
  exit_syscall(42);
}
