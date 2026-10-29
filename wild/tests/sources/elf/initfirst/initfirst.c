//#AbstractConfig:common
//#CompArgs:-fPIC
//#DiffIgnore:section.got

//#Config:shared:common
//#LinkArgs:-shared -z initfirst
//#CompArgs:-DSHARED
//#Mode:dynamic
//#RunEnabled:false
//#ExpectDynamic:DT_FLAGS_1

//#Config:exec:common
//#Object:runtime.c
//#LinkArgs:-pie -z initfirst
//#Mode:dynamic

#include "../common/runtime.h"

#ifdef SHARED
int foo(void) { return 42; }
#else
void _start(void) {
  runtime_init();
  exit_syscall(42);
}
#endif
