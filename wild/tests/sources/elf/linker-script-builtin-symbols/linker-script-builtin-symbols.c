//#AbstractConfig:default
//#Object:runtime.c
//#ReferenceLinkers:lld
//#CompArgs:-fno-pie
//#LinkArgs:-static

//#Config:no_sections:default
//#LinkerScript:builtin-symbols.ld
//#LinkerScript:always_emitted.ld
//#ExpectSym:_etext

//#Config:with_sections_defined:default
//#LinkerScript:always_emitted.ld
//#LinkerScript:section-table.ld
//#DiffEnabled:false
//#RunEnabled:false

//#Config:with_sections_undefined:default
//#LinkerScript:builtin-symbols.ld
//#LinkerScript:section-table.ld
//#ExpectError:(?i-u)symbol( not found:)? '?_etext'?
//#ExpectErrorWild:Symbol '_etext' referenced by linker script does not exist

//#Config:with_sections_undefined_tdata_start:default
//#LinkerScript:section-table.ld
//#LinkArgs:--defsym=ts=__tdata_start
//#ExpectError:(?i-u)symbol( not found:)? '?__tdata_start'?
//#ExpectErrorWild:Symbol '__tdata_start' referenced by --defsym does not exist

//#Config:no_sections_riscv:no_sections
//#Arch:riscv64,riscv32
//#ExpectSym:__global_pointer$

#include "../common/runtime.h"

void _start() {
  runtime_init();
  exit_syscall(42);
}
