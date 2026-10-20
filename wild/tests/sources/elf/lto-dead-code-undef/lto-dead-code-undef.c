// Tests that we don't error if there's an undefined symbol in dead code, when the dead code isn't
// referenced from outside the IR.

//#AbstractConfig:default
//#Object:runtime.c
//#RequiresLinkerPlugin:true
//#CompArgs:-flto -O1

//#Config:gcc:default
// LLD does not support GCC LTO plugins.
//#ReferenceLinkers:bfd
//#LinkerDriver:gcc
//#Object:lto-dead-code-undef-1.c
//#LinkArgs:-flto -O1 -nostdlib -Wl,-z,now
//#DiffIgnore:section.rodata
//#DiffIgnore:section.got
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:llvm
//#RequiresLlvmLto:true
// LLD fails to define .TOC. when it is first referenced by LTO-generated code.
//#SkipArch:ppc64le
//#Compiler:clang
//#CompArgs:-flto -O2 -fno-pie
//#LinkArgs:-no-pie
//#ReferenceLinkers:lld
//#Object:runtime.c:-fno-lto
//#Object:lto-dead-code-undef-1.c

//#Config:llvm-thin:llvm
//#CompArgs:-flto=thin

#include "../common/runtime.h"

int foo(int x);

void _start(void) {
  runtime_init();
  exit_syscall(foo(0));
}
