//#AbstractConfig:default
//#SkipArch:ppc64le
//#RequiresGlibc:true
//#Mode:dynamic
//#Object:runtime.c
//#LinkArgs:-z now
//#LinkSoArgs:-z now
//#Shared:ifunc-interposition-lib.c
//#ReferenceLinkers:bfd,lld
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:.dynamic.DT_RELA*
//#DiffIgnore:section.rodata

//#Config:plt:default
//#CompSoArgs:-fPIC

//#Config:got:default
//#CompSoArgs:-fPIC -fno-plt

#include "../common/runtime.h"

typedef int (*Func)(void);
int call_foo(void);
Func get_foo(void);

int foo(void) { return 42; }

void _start(void) {
  runtime_init();
  if (call_foo() != 42) {
    exit_syscall(1);
  }
  if (get_foo() != foo) {
    exit_syscall(2);
  }
  exit_syscall(42);
}
