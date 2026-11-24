//#AbstractConfig:base
//#Compiler:clang
//#RequiresLlvmLto:true
// LLD fails to define .TOC. when it is first referenced by LTO-generated code.
//#SkipArch:ppc64le
//#ReferenceLinkers:lld
//#CompArgs:-flto -O2 -fno-pie
//#Object:runtime.c:-fno-lto
//#Object:native.c:-fno-lto
//#LinkArgs:-z now -no-pie

//#Config:full:base
//#Object:answer.c

//#Config:thin:full
//#CompArgs:-flto=thin

//#Config:full-ppc64le:full
//#Arch:ppc64le
// LLD fails to define .TOC.; execution checks the linked program instead.
//#ReferenceLinkers:

//#Config:thin-ppc64le:full-ppc64le
//#CompArgs:-flto=thin

//#Config:archive:base
//#Archive:answer.c
//#Archive:unused.c

//#Config:thin-archive:base
//#CompArgs:-flto=thin
//#ThinArchive:answer.c
//#ThinArchive:unused.c

//#Config:pie:full
//#CompArgs:-fPIE
//#LinkArgs:-z now -pie

//#Config:shared:full
//#CompArgs:-fPIC
//#LinkArgs:-z now -shared
//#RunEnabled:false
//#ExpectDynSym:answer
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT
//#DiffIgnore:section.got.plt.entsize

//#Config:opt0:full
//#LinkArgs:-z now --lto-O0 --thinlto-jobs=1

//#Config:cache:thin
//#LinkArgs:--thinlto-cache-dir=$OUT_DIR/cache --thinlto-cache-policy=cache_size_files=100 --thinlto-jobs=1

//#Config:partial:base
// Wild's -r output loses the unnamed .eh_frame symbols produced by LLVM 18 on RISC-V.
//#Relocatable:answer.c:-fno-asynchronous-unwind-tables

//#Config:mixed:full
//#Object:unused.c:-flto=thin

//#Config:defsym:base
//#Object:override.c
//#LinkArgs:--defsym=answer=replacement

//#Config:defsym-thin:defsym
//#CompArgs:-flto=thin

//#Config:script:base
//#Object:override.c
//#AugmentLinkerScript:override.ld

//#Config:script-thin:script
//#CompArgs:-flto=thin

//#Config:forced:full
//#Object:retained.c
//#LinkArgs:--undefined=retained
//#ExpectSym:retained

//#Config:forced-thin:forced
//#CompArgs:-flto=thin

//#Config:bad-opt:full
//#ReferenceLinkers:
//#LinkArgs:--lto-O=9
//#ExpectError:LTO optimization level must be between 0 and 3

//#Config:bad-jobs:full
//#ReferenceLinkers:
//#LinkArgs:--thinlto-jobs=invalid
//#ExpectError:Invalid ThinLTO jobs value

//#Config:bad-policy:thin
//#ReferenceLinkers:
//#LinkArgs:--thinlto-cache-policy=not-a-policy
//#ExpectError:LLVM [0-9]+.*(policy|unknown|Unknown)

//#Config:explicit-plugin:full
//#ReferenceLinkers:
//#LinkArgs:--plugin=/does/not/exist
//#ExpectError:(No such file or directory|without linker-plugin support|linker plugin was not supplied)

#include "../common/runtime.h"

int answer(void);
int native(void);

void _start(void) {
  runtime_init();
  exit_syscall(answer() + native());
}
