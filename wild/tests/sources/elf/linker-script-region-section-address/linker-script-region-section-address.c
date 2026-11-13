//#LinkerScript:regions.ld
//#Object:runtime.c
//#ReferenceLinkers:bfd,lld
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables -fno-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections --no-relax

#include "../common/runtime.h"

int data1 __attribute__((section(".data1"))) = 1;
int data2 __attribute__((section(".data2"))) = 2;

void _start(void) {
  runtime_init();
  if (data1 != 1) {
    exit_syscall(1);
  }
  if (data2 != 2) {
    exit_syscall(2);
  }
  exit_syscall(42);
}
