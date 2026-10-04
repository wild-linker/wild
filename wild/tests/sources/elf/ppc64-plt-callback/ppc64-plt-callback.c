//#Config:default
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

//#Config:read-only:default
//#CompArgs:-DREAD_ONLY_CALLBACK
//#Object:pointer.s
//#ExpectSym:callback section=".rodata"

#include "../common/runtime.h"

typedef int (*Callback)(void);

static int implementation(void) { return 42; }
static Callback resolver(void) { return implementation; }
int selected(void) __attribute__((ifunc("resolver"), visibility("hidden")));

int invoke(Callback callback);

#ifdef READ_ONLY_CALLBACK
extern Callback const callback;
#endif

void _start(void) {
  runtime_init();
#ifdef READ_ONLY_CALLBACK
  exit_syscall(invoke(callback));
#else
  exit_syscall(invoke(selected));
#endif
}
