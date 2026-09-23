//#SkipArch:ppc64le
//#Mode:dynamic
//#ReferenceLinkers:lld
//#Object:runtime.c
//#Shared:shared.c
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:section.got.plt.entsize

#include <stdint.h>

#include "../common/runtime.h"

__asm__(
    ".pushsection .data\n"
    ".globl p\n"
    "p: .long f\n"
    ".balign 8\n"
    ".globl full_p\n"
    "full_p: .quad f\n"
    ".popsection\n");

typedef int (*Function)(void);

extern uint32_t p;
extern Function full_p;

#ifdef __x86_64__
__asm__(
    ".pushsection .data\n"
    ".globl signed_p\n"
    "signed_p:\n"
    ".reloc signed_p, R_X86_64_32S, f\n"
    ".long 0\n"
    ".popsection\n");

extern int32_t signed_p;
#endif

void _start(void) {
  runtime_init();
  Function ptr = (Function)(uintptr_t)p;

  if (ptr != full_p) {
    exit_syscall(1);
  }

#ifdef __x86_64__
  Function signed_ptr = (Function)(intptr_t)signed_p;
  if (signed_ptr != ptr) {
    exit_syscall(2);
  }
  if (signed_ptr() != 7) {
    exit_syscall(3);
  }
#endif

  exit_syscall(ptr() + 35);
}
