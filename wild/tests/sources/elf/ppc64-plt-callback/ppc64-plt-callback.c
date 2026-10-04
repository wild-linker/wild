//#Arch:ppc64le
//#Mode:dynamic
//#Object:runtime.c
//#CompArgs:-fno-pie
//#LinkArgs:-no-pie -z now
//#Shared:invoke.c
//#SoSingleLinker:lld
// LLD 21 emits a TOC-relative entry for address-taken IFUNCs.
//#ReferenceLinkers:bfd
//#DiffEnabled:false
//#RequiresGlibc:true

#include "../common/runtime.h"

typedef int (*Callback)(void);

static int implementation(void) { return 42; }
static Callback resolver(void) { return implementation; }
int selected(void) __attribute__((ifunc("resolver"), visibility("hidden")));

int invoke(Callback callback);

void _start(void) {
  runtime_init();
  exit_syscall(invoke(selected));
}
