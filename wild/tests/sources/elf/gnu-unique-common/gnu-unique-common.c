//#AbstractConfig:base
//#RequiresGlibc:true
//#Object:runtime.c
//#CompArgs:-fPIE
//#LinkArgs:-no-pie
//#ExpectSym:uniq type=object,size=4

//#Config:unique-first:base
//#Object:unique.cc
//#Object:common.c:-fcommon

//#Config:common-first:base
//#Object:common.c:-fcommon
//#Object:unique.cc

//#Config:larger-common-first:base
//#Object:common-large.c:-fcommon
//#Object:unique.cc

// The partial link sees the larger common symbol first.
//#Config:partial-larger-common:base
//#Relocatable:common-large.c:-fcommon,unique.cc

#include "../common/runtime.h"

extern int read_unique(void);

void _start(void) {
  runtime_init();

  if (read_unique() != 31) {
    exit_syscall(read_unique());
  }

  exit_syscall(42);
}
