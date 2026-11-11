// References within the object defining a weak symbol must keep the overriding LTO definition.
//#AbstractConfig:base
//#RequiresLinkerPlugin:true
//#CompArgs:-O2 -fno-lto -fno-pie -ffunction-sections
//#LinkArgs:-flto -static -nostdlib -Wl,-z,now
//#Object:runtime.c
//#Archive:unused.c
//#NoSym:unused_provider

//#AbstractConfig:clang:base
//#Compiler:clang
//#LinkerDriver:clang
// RISC-V LTO objects have an empty .text aligned to 4, but live code aligned to 2.
//#DiffIgnore:section.text.alignment

//#AbstractConfig:gcc:base
//#Compiler:gcc
//#LinkerDriver:gcc
// LLD does not support GCC LTO plugins.
//#ReferenceLinkers:bfd

//#Config:clang-weak-first:clang
//#Object:weak.c
//#Object:strong.c:-flto

//#Config:clang-strong-first:clang
//#Object:strong.c:-flto
//#Object:weak.c

//#Config:clang-weak-first-ref:clang-weak-first
//#CompArgs:-DDIRECT_REF

//#Config:clang-strong-first-ref:clang-strong-first
//#CompArgs:-DDIRECT_REF

//#Config:clang-ir-weak-first-ref:clang
//#CompArgs:-DDIRECT_REF
//#Object:weak.c:-flto
//#Object:strong.c:-flto

//#Config:clang-native-archive:clang
//#Archive:weak.c
//#Object:strong.c:-flto

//#Config:clang-lto-archive:clang
//#CompArgs:-DLOAD_STRONG
//#Object:weak.c
//#Archive:strong.c:-flto

//#Config:clang-unloaded-lto-archive:clang
//#CompArgs:-DEXPECTED=7
//#Object:weak.c
//#Archive:strong.c:-flto
//#NoSym:strong_anchor

//#Config:gcc-weak-first:gcc
//#Object:weak.c
//#Object:strong.c:-flto

//#Config:gcc-strong-first:gcc
//#Object:strong.c:-flto
//#Object:weak.c

//#Config:gcc-weak-first-ref:gcc-weak-first
//#CompArgs:-DDIRECT_REF

//#Config:gcc-strong-first-ref:gcc-strong-first
//#CompArgs:-DDIRECT_REF

//#Config:gcc-ir-weak-first-ref:gcc
//#CompArgs:-DDIRECT_REF
//#Object:weak.c:-flto
//#Object:strong.c:-flto

//#Config:gcc-native-archive:gcc
//#Archive:weak.c
//#Object:strong.c:-flto

//#Config:gcc-lto-archive:gcc
//#CompArgs:-DLOAD_STRONG
//#Object:weak.c
//#Archive:strong.c:-flto

//#Config:gcc-unloaded-lto-archive:gcc
//#CompArgs:-DEXPECTED=7
//#Object:weak.c
//#Archive:strong.c:-flto
//#NoSym:strong_anchor

#include "../common/runtime.h"

#ifndef EXPECTED
#define EXPECTED 42
#endif

int target(void);
int weak_fn(void);
int strong_anchor(void);

void _start(void) {
  runtime_init();
#ifdef LOAD_STRONG
  if (strong_anchor() != 0) {
    exit_syscall(100);
  }
#endif
  if (target() != EXPECTED) {
    exit_syscall(101);
  }
#ifdef DIRECT_REF
  if (weak_fn() != EXPECTED) {
    exit_syscall(102);
  }
#endif
  exit_syscall(42);
}
