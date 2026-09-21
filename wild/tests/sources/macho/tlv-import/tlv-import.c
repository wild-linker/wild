// Older versions of ld and current lld use `__thread_ptrs` for imported TLV descriptors,
// but the latest version of ld (Xcode 26.6) uses a GOT entry, whose behavior we match.
//#LinkerDriver:clang
//#SoSingleLinker:lld
//#ReferenceLinkers:ld
//#Shared:foo.c
//#Object:runtime.c
//#ExpectSection:__got
//#NoSection:__thread_ptrs

#include "../common/runtime.h"

extern _Thread_local int value;

int main(void) { exit_syscall(value + 41); }
