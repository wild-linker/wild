//#Config:mapped-headers
//#Arch:x86_64
//#ReferenceLinkers:bfd,lld
//#Object:runtime.c
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables -fno-unwind-tables
//#LinkArgs:-no-pie -e _start
//#LinkerScript:mapped-headers.ld
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text],offset=0,vaddr=0x20000,file-size=0x2000,mem-size=0x2000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data],vaddr=0x400000,file-size=4,mem-size=4
//#ExpectSym:__ehdr_start address=0x20000
//#ExpectSym:_start address=0x21000
//#ExpectSym:var1 address=0x400000

#include "../common/runtime.h"

static int var1 __attribute__((used, section(".data.ram1"))) = 0x01;

extern const unsigned char __ehdr_start[];

void _start(void) {
  runtime_init();
  if (__ehdr_start[0] != 0x7f || __ehdr_start[1] != 'E' || __ehdr_start[2] != 'L' ||
      __ehdr_start[3] != 'F') {
    exit_syscall(1);
  }
  exit_syscall(42);
}
