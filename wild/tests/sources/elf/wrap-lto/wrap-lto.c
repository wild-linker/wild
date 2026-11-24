//#AbstractConfig:default
//#RequiresLinkerPlugin:true
//#Object:runtime.c
//#Object:wrap-lto-2.c:-fno-lto
//#ReferenceLinkers:
//#CompArgs:-flto
//#LinkArgs:-flto -nostdlib -z now -Wl,-wrap,foo

//#Config:gcc:default
//#LinkerDriver:gcc
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:clang:default
//#Compiler:clang
//#LinkerDriver:clang
//#ReferenceLinkers:lld
//#SkipArch:ppc64le
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:clang-ppc64le:clang
//#Arch:ppc64le
// LLD 21 fails to define .TOC. when linking this LTO input.
//#ReferenceLinkers:bfd

//#Config:llvm
//#DiffIgnore:rel.R_X86_64_PC32.R_X86_64_PC32
//#Compiler:clang
//#RequiresLlvmLto:true
// LLD fails to define .TOC. when it is first referenced by LTO-generated code.
//#SkipArch:ppc64le
//#ReferenceLinkers:lld
//#Object:runtime.c:-fno-lto
//#CompArgs:-flto -O1 -fPIE
//#LinkArgs:-z now -pie
//#Object:wrap-lto-2.c:-fno-lto
//#LinkArgs:--wrap=foo

//#Config:llvm-thin:llvm
//#CompArgs:-flto=thin

#include "../common/runtime.h"

int foo(void);
int __real_foo(void);

int __wrap_foo(void) { return __real_foo() + 32; }

void _start(void) {
  runtime_init();
  if (foo() != 42) {
    exit_syscall(100);
  }
  exit_syscall(42);
}
