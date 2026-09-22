//#AbstractConfig:default
//#Object:runtime.c
//#CompArgs:-g -fmerge-constants -fPIC
//#ReferenceLinkers:

//#Config:only-keep-debug:default
//#LinkArgs:--only-keep-debug
//#RunEnabled:false
//#DiffEnabled:false
//#TestOnlyKeepDebug:true
//#ExpectSym:_start
//#ExpectSym:exit_syscall
//#ExpectSection:.symtab

//#Config:only-keep-debug-compressed:default
//#LinkArgs:--only-keep-debug --compress-debug-sections=zlib
//#RunEnabled:false
//#DiffEnabled:false
//#TestOnlyKeepDebug:true
//#ExpectSym:_start

//#Config:only-keep-debug-build-id:default
//#LinkArgs:--only-keep-debug --build-id=0x123456789abcdef0
//#RunEnabled:false
//#DiffEnabled:false
//#TestOnlyKeepDebug:true
//#ExpectSym:_start

//#Config:only-keep-debug-dynamic:default
//#Mode:dynamic
//#Shared:shared.c
//#LinkArgs:--only-keep-debug -z now
//#RunEnabled:false
//#DiffEnabled:false
//#TestOnlyKeepDebug:true
//#ExpectSym:_start

//#Config:only-keep-debug-shared:default
//#SkipArch: ppc64le
//#Mode:dynamic
//#LinkArgs:-shared --only-keep-debug
//#RunEnabled:false
//#DiffEnabled:false
//#TestOnlyKeepDebug:true
//#ExpectSym:_start

//#Config:only-keep-debug-copy-relocations:only-keep-debug-dynamic
//#Arch:x86_64,aarch64,riscv64
//#CompArgs:-g -fno-pic -fno-pie -DCOPY_RELOCATION
//#CompSoArgs:-fPIC
//#LinkArgs:-z now
//#WildExtraLinkArgs:--only-keep-debug
//#SoSingleLinker:lld
//#ExpectSym:shared_data section=".bss"

//#Config:only-keep-debug-got-plt-syms:only-keep-debug-dynamic
//#SkipArch:ppc64le
//#LinkArgs:-z now
//#WildExtraLinkArgs:--only-keep-debug --got-plt-syms
//#SoSingleLinker:lld
//#ExpectSym:shared_func$got

//#Config:only-keep-debug-android-relr:only-keep-debug
//#LinkArgs:--only-keep-debug -pie --pack-dyn-relocs=relr --use-android-relr-tags
//#ExpectSection:.relr.dyn type=8

//#Config:only-keep-debug-static:only-keep-debug
//#LinkArgs:-static --only-keep-debug

#include "../common/runtime.h"

int global_var = 42;

const char* msg1 = "non-debug merged string";
const char* msg2 = "non-debug merged string";

__attribute__((weak)) int shared_func(int x);

#ifdef COPY_RELOCATION
extern int shared_data;
#endif

void _start(void) {
  runtime_init();
  int val = global_var;
#ifdef COPY_RELOCATION
  val += shared_data;
#endif
  if (shared_func) {
    val = shared_func(val);
  }
  if (msg1[0] == msg2[0]) {
    val += 1;
  }
  exit_syscall(val);
}
