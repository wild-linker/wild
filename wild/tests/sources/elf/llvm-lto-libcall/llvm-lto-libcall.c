//#Config:full
//#Compiler:clang
//#RequiresLlvmLto:true
//#ReferenceLinkers:lld
//#CompArgs:-O2 -fno-lto -fno-pie -fno-stack-protector
//#LinkArgs:-no-pie
//#Object:runtime.c
//#Object:copy.c:-flto
//#Archive:memcpy.c:-flto -ffreestanding

//#Config:thin
//#Compiler:clang
//#RequiresLlvmLto:true
//#ReferenceLinkers:lld
//#CompArgs:-O2 -fno-lto -fno-pie -fno-stack-protector
//#LinkArgs:-no-pie
//#Object:runtime.c
//#Object:copy.c:-flto=thin
//#Archive:memcpy.c:-flto=thin -ffreestanding

#include "../common/runtime.h"

void copy(void*, const void*, unsigned long);

void _start(void) {
  static char source[1024] = {42};
  char destination[1024];
  copy(destination, source, sizeof(source));
  exit_syscall(destination[0]);
}
