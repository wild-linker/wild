//#AbstractConfig:base
//#Arch:x86_64
//#RequiresLinkerPlugin:true
//#Object:runtime.c
//#Object:other.cc
//#CompArgs:-flto -O1
//#LinkArgs:-flto -nostdlib -Wl,-z,now,--gc-sections
//#DiffIgnore:section.got

//#Config:gcc:base
//#Compiler:gcc
//#LinkerDriver:gcc

//#Config:gcc-mixed:gcc
//#Object:native.cc:-fno-lto

//#Config:clang:base
//#Compiler:clang
//#LinkerDriver:clang
//#DiffIgnore:section.eh_frame.type

//#Config:clang-mixed:clang
//#Object:native.cc:-fno-lto

#include "value.h"

extern "C" {
#include "../common/runtime.h"
}

int other();
extern "C" void _start() {
  runtime_init();
  exit_syscall(value() + other());
}
