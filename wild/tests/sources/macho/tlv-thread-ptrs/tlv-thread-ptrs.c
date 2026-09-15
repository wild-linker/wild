//#LinkerDriver:clang
//#SoSingleLinker:lld
//#Shared:foo.c
//#Object:runtime.c
//#DiffIgnore:section.__unwind_info
//#ExpectSection:__thread_ptrs

#include "../common/runtime.h"

extern _Thread_local int value;

int main(void) { exit_syscall(value + 41); }
