//#Config:default
//#Object:runtime.c
//#LinkArgs:-e _custom_entry
//#ExpectEntry:_custom_entry

#include "../common/runtime.h"

void main(void) { exit_syscall(1); }

void custom_entry(void) { exit_syscall(42); }
