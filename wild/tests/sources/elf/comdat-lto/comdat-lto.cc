//#AbstractConfig:base
//#Arch:x86_64,aarch64
//#RequiresLinkerPlugin:true
//#Object:runtime.c
//#Object:other.cc
//#CompArgs:-flto -O1
//#LinkArgs:-flto -nostdlib -Wl,-z,now
//#DiffIgnore:section.got

//#Config:gcc:base
// LLD does not support GCC LTO plugins.
//#ReferenceLinkers:bfd
//#Compiler:gcc
//#LinkerDriver:gcc
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:gcc-mixed:gcc
//#Object:native.cc:-fno-lto

//#Config:clang:base
//#Compiler:clang
//#LinkerDriver:clang
//#DiffIgnore:section.eh_frame.type
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:clang-mixed:clang
//#Object:native.cc:-fno-lto

//#Config:llvm
//#Compiler:clang
//#RequiresLlvmLto:true
// LLD fails to define .TOC. when it is first referenced by LTO-generated code.
//#SkipArch:ppc64le
//#ReferenceLinkers:lld
//#Object:runtime.c:-fno-lto
//#CompArgs:-flto -O1 -fno-pie
//#LinkArgs:-z now -no-pie
//#Object:other.cc
//#Object:native.cc:-fno-lto

//#Config:llvm-thin:llvm
//#CompArgs:-flto=thin

#include "value.h"

extern "C" {
#include "../common/runtime.h"
}

int other();
extern "C" void _start() {
  runtime_init();
  exit_syscall(value() + other());
}
