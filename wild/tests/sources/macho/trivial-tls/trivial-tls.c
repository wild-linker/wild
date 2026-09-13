//#Object:runtime.c
//#DiffIgnore:section.__unwind_info
//#ExpectSection:__thread_data
//#ExpectSection:__thread_bss
//#ExpectSection:__thread_vars

#include "../common/runtime.h"

_Thread_local int initialized = 1;
_Thread_local int uninitialized __attribute__((aligned(64)));

void main(void) { exit_syscall(initialized + uninitialized + 41); }
