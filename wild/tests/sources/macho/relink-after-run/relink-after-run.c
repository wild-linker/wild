//#Object:runtime.c
//#TestRelinkAfterRun:true
//#WildExtraLinkArgs:--threads=3
//#InMemoryFileSystem:false

#include "../common/runtime.h"

void main(void) { exit_syscall(42); }
