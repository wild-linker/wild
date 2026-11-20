//#AbstractConfig:default
//#Object:runtime.c
//#Object:sections.s
//#ExpectSym:foo alignment=0x10000
//#ExpectSym:data_marker alignment=0x20000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:gc:default
//#NoSym:unused_marker

//#Config:no-gc:default
//#LinkArgs:--no-gc-sections
//#ExpectSym:unused_marker alignment=0x100000

#include "../common/runtime.h"

extern char data_marker[];
extern unsigned char live_data;

void _start(void) {
  runtime_init();

  if ((unsigned long)data_marker % 0x20000 != 0) {
    exit_syscall(1);
  }

  if (live_data != 42) {
    exit_syscall(2);
  }

  exit_syscall(42);
}
