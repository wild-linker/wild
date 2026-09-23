//#AbstractConfig:default
//#Object:runtime.c
//#TestHardLinks:true

//#Config:mmap:default
//#WildExtraLinkArgs:--threads=3 --mmap-output-file

//#Config:buffered:default
//#WildExtraLinkArgs:--threads=3 --no-mmap-output-file

#include "../common/runtime.h"

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
