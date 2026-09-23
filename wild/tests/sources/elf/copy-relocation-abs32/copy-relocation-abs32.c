//#Mode:dynamic
//#ReferenceLinkers:lld
//#Object:runtime.c
//#Shared:shared.s
//#DiffIgnore:.dynamic.DT_NEEDED

#include <stdint.h>

#include "../common/runtime.h"

__asm__(
    ".pushsection .data\n"
    ".globl p\n"
    "p: .long v\n"
    ".balign 8\n"
    ".globl full_p\n"
    "full_p: .quad v\n"
    ".popsection\n");

extern uint32_t p;
extern int* full_p;

#ifdef __x86_64__
__asm__(
    ".pushsection .data\n"
    ".globl signed_p\n"
    "signed_p:\n"
    ".reloc signed_p, R_X86_64_32S, v\n"
    ".long 0\n"
    ".popsection\n");

extern int32_t signed_p;
#endif

void _start(void) {
  runtime_init();
  int* ptr = (int*)(uintptr_t)p;

  if (ptr != full_p) {
    exit_syscall(1);
  }

#ifdef __x86_64__
  if ((int*)(intptr_t)signed_p != ptr) {
    exit_syscall(2);
  }
#endif

  exit_syscall(*ptr + 35);
}
