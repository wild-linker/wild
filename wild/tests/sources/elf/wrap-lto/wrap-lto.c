//#AbstractConfig:default
//#RequiresLinkerPlugin:true
//#Object:runtime.c
//#Object:wrap-lto-2.c:-fno-lto
//#ReferenceLinkers:
//#CompArgs:-flto
//#LinkArgs:-flto -nostdlib -z now -Wl,-wrap,foo

//#Config:gcc:default
//#LinkerDriver:gcc

//#Config:clang:default
//#Compiler:clang
//#LinkerDriver:clang
//#ReferenceLinkers:lld
//#SkipArch:ppc64le
// RISC-V LTO objects have an empty .text aligned to 4, but live code aligned to 2.
//#DiffIgnore:section.text.alignment

//#Config:clang-ppc64le:clang
//#Arch:ppc64le
// LLD 21 fails to define .TOC. when linking this LTO input.
//#ReferenceLinkers:bfd

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
