// Wild and BFD use an absolute move where LLD uses a PC-relative lea.
//#DiffIgnore:rel.different-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).MovIndirectToLea.static-non-pie
// This test does stuff with some of the linker-defined symbols. These symbols
// are generally used by libc.

//#Object:runtime.c
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

struct Rela {
  long long a, b, c;
};

extern const struct Rela __rela_iplt_start __attribute__((weak));
extern const struct Rela __rela_iplt_end __attribute__((weak));

void _start(void) {
  runtime_init();

  int value = 42;
  // We shouldn't have any .rela.plt entries, so this loop should terminate
  // without dereferencing any RELA entries.
  for (const struct Rela* e = &__rela_iplt_start; e < &__rela_iplt_end; ++e) {
    value += 1 + e->a;
  }
  exit_syscall(value);
}
