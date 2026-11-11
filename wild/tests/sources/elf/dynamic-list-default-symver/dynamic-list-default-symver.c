//#Config:exact
//#Object:runtime.c
//#Shared:lib.c
//#Mode:dynamic
//#RequiresGlibc:true
//#ReferenceLinkers:lld
//#CompArgs:-fPIE
//#CompSoArgs:-fPIC
//#LinkArgs:-z now --export-dynamic -no-pie
//#LinkSoArgs:-z now --version-script=./versions.map
//#LinkSoArgs:--dynamic-list=./exact.list
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:.dynamic.DT_RELA*
//#DiffIgnore:.dynamic.DT_JMPREL
//#DiffIgnore:.dynamic.DT_PLTRELSZ
//#DiffIgnore:.dynamic.DT_PLTGOT
//#DiffIgnore:section.got
//#DiffIgnore:section.plt
//#DiffIgnore:section.gnu.version_d.alignment
//#DiffIgnore:section.gnu.version_r.alignment
//#DiffIgnore:version_d.verdef_1
//#DiffIgnore:dynsym.__global_pointer$.section

#include "../common/runtime.h"

int f0(void) { return 11; }

int call_f0(void);

void _start(void) {
  runtime_init();
  if (call_f0() != 11) {
    exit_syscall(45);
  }
  exit_syscall(42);
}
