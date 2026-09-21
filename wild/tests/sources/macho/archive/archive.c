//#Object:runtime.c
//#Archive:from_archive.c
//#ExpectSym:_main

#include "../common/runtime.h"

int from_archive(void);

void main(void) { exit_syscall(from_archive()); }
