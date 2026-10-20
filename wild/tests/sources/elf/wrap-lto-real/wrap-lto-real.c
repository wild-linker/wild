//#AbstractConfig:default
//#RequiresLinkerPlugin:true
//#Object:runtime.c
//#Object:wrap-lto-real-def.c:-flto
//#Object:wrap-lto-real-wrap.c
//#ReferenceLinkers:
//#LinkArgs:-flto -nostdlib -z now -Wl,-wrap,foo
//#DiffIgnore:rel.referent-mismatch

//#Config:gcc:default
//#LinkerDriver:gcc
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:clang:default
//#Compiler:clang
//#LinkerDriver:clang
//#ReferenceLinkers:lld
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:llvm
//#DiffIgnore:rel.R_X86_64_PC32.R_X86_64_PC32
//#Compiler:clang
//#RequiresLlvmLto:true
//#ReferenceLinkers:lld
//#Object:runtime.c:-fno-lto
//#CompArgs:-flto -O1 -fPIE
//#LinkArgs:-z now -pie
//#Object:wrap-lto-real-def.c
//#Object:wrap-lto-real-wrap.c:-fno-lto
//#LinkArgs:--wrap=foo

//#Config:llvm-thin:llvm
//#CompArgs:-flto=thin

#include "../common/runtime.h"

int foo(void);

void _start(void) {
  runtime_init();
  if (foo() != 42) {
    exit_syscall(100);
  }
  exit_syscall(42);
}
