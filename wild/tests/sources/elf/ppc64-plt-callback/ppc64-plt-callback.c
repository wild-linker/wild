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
//#CompSoArgs:-fPIE

//#Config:read-only:default
//#CompArgs:-DREAD_ONLY_CALLBACK
//#Object:pointer.s
//#ExpectSym:callback section=".rodata"

//#Config:warn-textrel:read-only
//#SoSingleLinker:ld
//#LinkArgs:--warn-textrel
//#ExpectWarning:output contains dynamic relocations in read-only sections|creating.*DT_TEXTREL

//#Config:error-textrel:read-only
//#LinkArgs:-z notext -z text
//#ExpectError:output contains dynamic relocations in read-only sections \(-z text\)|read-only segment has dynamic relocations

//#Config:error-warn-textrel:warn-textrel
//#LinkArgs:-z text --warn-textrel

//#Config:warn-error-textrel:error-textrel
//#SoSingleLinker:ld
//#LinkArgs:--warn-textrel -z text

//#Config:allow-textrel:read-only
//#LinkArgs:-z text -z notext
//#ExpectWarningWild:^$

//#Config:warn-allowed-textrel:read-only
//#SoSingleLinker:ld
//#LinkArgs:--warn-textrel -z notext
//#ExpectWarningWild:^$

//#Config:textoff:read-only
//#SoSingleLinker:ld
//#LinkArgs:-z text --warn-textrel -z textoff
//#ExpectWarningWild:^$

//#Config:warn-textrel-old-dtags:warn-textrel
//#LinkArgs:--disable-new-dtags
//#ExpectDynamic:DT_TEXTREL

//#Config:no-textrel:default
//#SoSingleLinker:ld
//#LinkArgs:-z text --warn-textrel
//#ExpectWarning:^$

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
