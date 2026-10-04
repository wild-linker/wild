//#RequiresGlibc:true
// BFD rejects these different default versions with an unresolvable PLT relocation.
//#ReferenceLinkers:lld
//#Object:runtime.c
//#Mode:dynamic
//#Shared:lib-v1.c
//#Shared:lib-v2.c
//#Shared:lib-v2-again.c
//#CompSoArgs:-fPIC
//#LinkArgs:--version-script=./versions.map
//#SoSingleLinker:lld
//#WildExtraLinkArgs:--threads=1
//#TestUpdateInPlace:true
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT
//#DiffIgnore:section.gnu.version_d.alignment
//#DiffIgnore:section.gnu.version_r.alignment
//#DiffIgnore:section.got.plt.entsize
//#DiffIgnore:version_d.verdef_1

#include "../common/runtime.h"

int foo_v2(void);
__asm__(".symver foo_v2,foo@V2");

void _start(void) {
  runtime_init();
  if (foo_v2() != 22) {
    exit_syscall(1);
  }
  exit_syscall(42);
}
