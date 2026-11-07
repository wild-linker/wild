//#LinkerScript:sections.ld
//#ReferenceLinkers:bfd,lld
//#SkipArch:riscv64
//#Object:runtime.c
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections
//#ExpectProgramHeader:LOAD sections=[*,.data,.orphan_data,.bss]
//#ExpectSym:data_var section=.data
//#ExpectSym:orphan_var section=.orphan_data
//#ExpectSym:bss_var section=.bss

#include "../common/runtime.h"

int data_var = 42;
int __attribute__((section(".orphan_data"))) orphan_var = 100;
int bss_var;

void _start(void) {
  if (data_var != 42 || orphan_var != 100 || bss_var != 0) {
    exit_syscall(1);
  }
  orphan_var = 101;
  if (orphan_var != 101) {
    exit_syscall(2);
  }
  exit_syscall(42);
}
