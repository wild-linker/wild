//#AbstractConfig:base
//#Compiler:clang
//#RequiresLlvmLto:true
//#ReferenceLinkers:lld
// The native foo call targets __wrap_foo, which linker-diff cannot match by name.
//#DiffIgnore:rel.referent-mismatch
//#SkipArch:ppc64le
//#CompArgs:-O2 -fno-lto -fno-pie
//#Object:runtime.c
//#LinkArgs:-no-pie --wrap=foo

//#Config:full:base
//#Object:definition.c:-flto

//#Config:thin:base
//#Object:definition.c:-flto=thin

//#Config:full-real:full
//#CompArgs:-DCALL_REAL

//#Config:thin-real:thin
//#CompArgs:-DCALL_REAL

#include "../common/runtime.h"

int alias(void);
int foo(void);
int __real_foo(void);

int __wrap_foo(void) {
#ifdef CALL_REAL
  return __real_foo() + 32;
#else
  return 42;
#endif
}

void _start(void) {
  runtime_init();

  if (alias() != 10 || foo() != 42) exit_syscall(100);

  exit_syscall(42);
}
