//#Config:header-overlap
//#RunEnabled:false
//#ReferenceLinkers:bfd,lld
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables -fno-unwind-tables
//#LinkArgs:-no-pie -e _start
//#LinkerScript:header-overlap.ld
//#NoProgramHeader:PHDR
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x10000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data,*],vaddr=0x400000
//#ExpectSym:_start address=0x10000
//#ExpectSym:var1 address=0x400000

//#Config:unmapped-ehdr-start:header-overlap
//#Arch:x86_64
//#Object:runtime.c
//#RunEnabled:true
//#CompArgs:-DCHECK_EHDR_START
// BFD leaves __ehdr_start undefined; LLD defines it at zero.
//#DiffMatchAny:true

static int var1 __attribute__((used, section(".data.ram1"))) = 0x01;

#ifdef CHECK_EHDR_START
#include "../common/runtime.h"

extern const char __ehdr_start[] __attribute__((weak));

void _start(void) {
  runtime_init();
  if (__ehdr_start != 0) {
    exit_syscall(1);
  }
  exit_syscall(42);
}
#else
void _start(void) {}
#endif
