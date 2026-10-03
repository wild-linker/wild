//#AbstractConfig:default
//#Object:runtime.c
//#CompArgs: -Wa,-L
//#ReferenceLinkers:bfd,lld

//#Config:discard-none:default
//#LinkArgs:--discard-none
//#ExpectSym:.Ltmp0 section=".text"
//#ExpectSym:local_func section=".text"
//#NoSym:unused

//#Config:discard-locals:default
//#LinkArgs:-X
//#NoSym:.Ltmp0
//#ExpectSym:local_func section=".text"

//#Config:discard-all:default
//#LinkArgs:-x
//#NoSym:.Ltmp0
//#NoSym:local_func
//#ExpectSym:_start section=".text"

#include "../common/runtime.h"

asm(".Ltmp0:");

static int unused = 4;
static int __attribute__((used, noinline)) local_func(void) { return 42; }

void _start(void) {
  runtime_init();
  if (local_func() != 42) {
    exit_syscall(1);
  }
  exit_syscall(42);
}
