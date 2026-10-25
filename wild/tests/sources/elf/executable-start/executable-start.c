//#AbstractConfig:default
//#Object:runtime.c
//#Object:ptr_black_box.c
//#ReferenceLinkers:bfd,lld
//#DiffMatchAny:true

//#Config:no-pie:default
//#LinkArgs:-no-pie -znow
//#CompArgs:-fPIE

//#Config:pie:default
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:dynamic:default
//#Mode:dynamic
//#Shared:force-dynamic-linking.c
//#DiffIgnore:.dynamic.DT_NEEDED
//#CompArgs:-fPIE
//#CompSoArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/ptr_black_box.h"
#include "../common/runtime.h"

extern char __executable_start;

void _start(void) {
  runtime_init();

  if (ptr_to_int(&__executable_start) > ptr_to_int(&_start)) {
    exit_syscall(10);
  }

  if (ptr_to_int(&__executable_start) == 0) {
    exit_syscall(11);
  }

  exit_syscall(42);
}
