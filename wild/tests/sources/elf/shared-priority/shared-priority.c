// Tests related to how we handle symbols that are defined in shared objects and also in other
// places like archives.

//#AbstractConfig:default
//#CompArgs:-fPIC
//#CompSoArgs:-fPIC
//#Object:runtime.c
//#Mode:dynamic
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT
//#DiffIgnore:section.got
// TODO: RISC-V BFD linker keeps the symbol in .dynsym section
//#DiffIgnore:section.rela.dyn
//#DiffIgnore:dynsym.var1.section

//#Config:shared-first-archive-not-loaded:default
//#Shared:shared-priority-1.c
//#Archive:shared-priority-2.c
//#LinkArgs:-no-pie

//#Config:archive-first:default
// Wild and BFD use an absolute move where LLD uses a PC-relative lea.
//#DiffIgnore:rel.different-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).MovIndirectToLea.dynamic-non-pie
//#Archive:shared-priority-1.c
//#Shared:shared-priority-2.c
//#LinkArgs:-no-pie

#include "../common/runtime.h"

int foo(void);

extern int var1;

void _start(void) {
  runtime_init();

  if (var1 != 65) {
    exit_syscall(101);
  }
  if (foo() != 10) {
    exit_syscall(100);
  }
  exit_syscall(42);
}
