//#AbstractConfig:default
//#Object:runtime.c

//#Config:stack-size:default
//#LinkArgs:-z stack-size=0x1000 -z now
// AArch64 BFD emits a GNU_PROPERTY_STACK_SIZE note for -z stack-size.
//#DiffIgnore:section.note.gnu.property
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

void _start() {
  runtime_init();
  exit_syscall(42);
}
