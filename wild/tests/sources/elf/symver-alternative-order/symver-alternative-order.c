// This test verifies that an unversioned definition from an archive overrides multiple versioned
// definitions from a shared object. Via TestUpdateInPlace, it also verifies that output is
// deterministic for this case even when thread count differs.

//#Config:default
//#RequiresGlibc:true
//#ReferenceLinkers:bfd,lld
//#Object:runtime.c
//#Mode:dynamic
//#Shared:lib.c
//#Archive:archive.c
//#CompSoArgs:-fPIC
//#LinkArgs:--version-script=./versions.map
//#SoSingleLinker:wild
//#WildExtraLinkArgs:--threads=1
//#TestUpdateInPlace:true
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT

//#Config:explicit:default
//#CompArgs:-DEXPLICIT_VERSION
// LLD emits a copy relocation for the explicit version instead of using the archive definition.
//#ReferenceLinkers:bfd
// BFD leaves the explicit version's AArch64 GOT entry for the dynamic loader to resolve.
//#DiffIgnore:rel.R_AARCH64_ADR_GOT_PAGE.R_AARCH64_ADR_GOT_PAGE

//#Config:repeated:default
//#Shared:lib2.c

#include "../common/runtime.h"

extern int cpu_model;
int pull_archive(void);
#ifdef EXPLICIT_VERSION
extern int cpu_model_v1;
__asm__(".symver cpu_model_v1,cpu_model@V1");
#endif

int versioned_v1(void);
int versioned_v2(void);
__asm__(".symver versioned_v1,versioned@V1");
__asm__(".symver versioned_v2,versioned@V2");

void _start(void) {
  runtime_init();
  if (cpu_model != 42 || pull_archive() != 42) {
    exit_syscall(1);
  }
#ifdef EXPLICIT_VERSION
  if (cpu_model_v1 != 42) {
    exit_syscall(3);
  }
#endif
  if (versioned_v1() != 1 || versioned_v2() != 2) {
    exit_syscall(2);
  }
  exit_syscall(42);
}
