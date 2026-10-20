//#AbstractConfig:base
//#RequiresLlvmLto:true
//#Compiler:clang
//#ReferenceLinkers:lld
//#CompArgs:-O2 -fcommon -fno-lto -fno-pic -fno-pie
//#Object:runtime.c
//#Object:init.c
//#ExpectSym:data size=4000
//#LinkArgs:-z nostart-stop-gc -no-pie
//#DiffIgnore:section.init_array.entsize

//#Config:full:base
//#Object:small.c:-flto
//#Object:large.c:-flto

//#Config:thin:base
//#Object:small.c:-flto=thin
//#Object:large.c:-flto=thin

#include "../common/init.h"
#include "../common/runtime.h"

extern int data[];
extern int initialized;
int call_override(void);

int override(void) { return 42; }

extern int missing(void) __attribute__((weak));
extern int __start_entries[];
extern int __stop_entries[];

void _start(void) {
  runtime_init();
  call_init_functions();

  if (initialized != 42 || call_override() != 42 || missing) exit_syscall(100);

  int sum = 0;

  for (int* entry = __start_entries; entry != __stop_entries; ++entry) sum += *entry;

  data[999] = sum;
  exit_syscall(data[999]);
}
