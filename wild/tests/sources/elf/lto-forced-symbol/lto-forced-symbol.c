//#AbstractConfig:base
//#RequiresLinkerPlugin:true
//#CompArgs:-O2 -fno-lto -fno-pie
//#LinkArgs:-flto -static -nostdlib -no-pie -Wl,-z,now,--undefined=retained
//#Object:runtime.c
//#ExpectSym:retained

//#AbstractConfig:gcc:base
//#Compiler:gcc
//#LinkerDriver:gcc
// LLD does not support GCC LTO plugins.
//#ReferenceLinkers:bfd

//#Config:gcc-object:gcc
//#Object:retained.c:-flto

//#Config:gcc-archive:gcc
//#Archive:retained.c:-flto

//#AbstractConfig:clang:base
//#Compiler:clang
//#LinkerDriver:clang
// RISC-V LTO objects have an empty .text aligned to 4, but live code aligned to 2.
//#DiffIgnore:section.text.alignment

//#Config:clang-object:clang
//#Object:retained.c:-flto

//#Config:clang-archive:clang
//#Archive:retained.c:-flto

//#Config:thin-object:clang
//#Object:retained.c:-flto=thin

//#Config:thin-archive:clang
//#Archive:retained.c:-flto=thin

//#Config:gcc-weak-first:gcc
//#Object:weak.c:-flto
//#Object:retained.c:-flto

//#Config:clang-weak-first:clang
//#Object:weak.c:-flto
//#Object:retained.c:-flto

//#Config:thin-weak-first:clang
//#Object:weak.c:-flto=thin
//#Object:retained.c:-flto=thin

#include "../common/runtime.h"

void _start(void) { exit_syscall(42); }
