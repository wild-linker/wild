//#Config:default
//#Object:runtime.c
//#Object:build-id-note.s
//#LinkArgs:--build-id=0x6767676767676767

//#Config:linker-script:default
//#Arch:x86_64
//#LinkerScript:build-id.ld
//#ExpectSectionBytes:.note.gnu.build-id=0x040000000800000003000000474e55006767676767676767

#include "../common/runtime.h"

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
