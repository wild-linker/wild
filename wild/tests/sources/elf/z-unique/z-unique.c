//#AbstractConfig:common
//#CompArgs:-fPIC
//#DiffIgnore:section.got
//#Mode:dynamic

//#Config:shared:common
//#LinkArgs:-shared -z unique
//#CompArgs:-DSHARED
//#Mode:unspecified
//#RunEnabled:false
//#ExpectDynamic:DT_GNU_FLAGS_1

//#Config:exec:common
//#CompSoArgs:-fno-pie
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT
//#Shared:runtime.c
//#LinkArgs:-no-pie -z unique
//#ExpectDynamic:DT_GNU_FLAGS_1

//#Config:pie:common
//#Object:runtime.c
//#LinkArgs:-pie -z unique
//#ExpectDynamic:DT_GNU_FLAGS_1

//#Config:static-pie:common
//#CompArgs:-fPIC
//#Object:runtime.c
//#LinkArgs:-pie --no-dynamic-linker -z unique
//#Mode:static
//#ExpectDynamic:DT_GNU_FLAGS_1

//#Config:static
//#CompArgs:-fno-pie
//#Object:runtime.c
//#LinkArgs:-no-pie -z unique
//#Mode:static

//#Config:disabled:common
//#Object:runtime.c
//#LinkArgs:-pie -z unique -z nounique -z unique

#include "../common/runtime.h"

#ifdef SHARED
int foo(void) { return 42; }
#else
void _start(void) {
  runtime_init();
  exit_syscall(42);
}
#endif
