//#LinkerScript:regions.ld
//#Object:runtime.c
//#ReferenceLinkers:bfd,lld
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables -fno-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections --no-relax

#include "../common/runtime.h"

volatile int bss;
// Exceed the page alignment on all supported architectures.
volatile int data __attribute__((aligned(262144))) = 42;

void _start(void) {
  runtime_init();
  if (data != 42) {
    exit_syscall(1);
  }
  if (bss != 0) {
    exit_syscall(2);
  }
  exit_syscall(42);
}
