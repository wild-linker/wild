// Checks that a common symbol declared in several files is allocated with the strictest alignment
// of its declarations, not just that of the largest one. pad is a 4-aligned common, so without
// that, arr could be placed right after it and misaligned.
//#AbstractConfig:default
//#Object:runtime.c
//#Object:pad.s
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:strict-first:default
//#Object:strict.s
//#Object:large.s

//#Config:large-first:default
//#Object:large.s
//#Object:strict.s

#include <stdint.h>

#include "../common/runtime.h"

extern char arr[];
extern char pad[];

void _start(void) {
  runtime_init();
  pad[0] = 1;
  exit_syscall((uintptr_t)arr % 64 == 0 ? 42 : 1);
}
