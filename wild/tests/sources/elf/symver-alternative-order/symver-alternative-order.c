// This test verifies that we produce deterministic output when we have an unversioned symbol with
// the same name as default versioned symbol. The determinism is verified by TestUpdateInPlace.

//#Object:runtime.c
//#Mode:dynamic
//#Shared:lib.c
//#Archive:archive.c
//#CompSoArgs:-fPIC
//#LinkArgs:--version-script=./versions.map
//#SoSingleLinker:wild
//#WildExtraLinkArgs:--threads=1
//#TestUpdateInPlace:true
// TODO: More work is needed. See #1302. Until we properly resolve which symbol is selected,
// there's not much point diffing.
//#DiffEnabled:false

#include "../common/runtime.h"

extern int cpu_model;
int pull_archive(void);

void _start(void) {
  runtime_init();
  if (cpu_model != 42 || pull_archive() != 42) {
    exit_syscall(1);
  }
  exit_syscall(42);
}
