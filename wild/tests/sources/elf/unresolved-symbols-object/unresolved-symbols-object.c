//#AbstractConfig:default
//#Object:runtime.c
//#SkipArch: loongarch64

/* BFD rejects the code on loongarch: relocation truncated to fit: R_LARCH_B26
   against symbol `foo'. */

//#Config:ignore-all-dynamic:default
// LLD retains dynamic PLT references to unresolved symbols; Wild and BFD resolve them to zero.
//#DiffMatchAny:true
//#Mode:dynamic
//#RunEnabled:false
//#Shared:force-dynamic-linking.c
//#LinkArgs:--unresolved-symbols=ignore-all -z now
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:section.got
//#CompArgs:-fPIE
//#CompSoArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:report-all:default
//#LinkArgs:--unresolved-symbols=report-all
//#ExpectError:foo
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:ignore-all:default
// LLD leaves GOT loads of unresolved symbols unrelaxed.
//#DiffIgnore:rel.extra-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).static-non-pie
//#LinkArgs:--unresolved-symbols=ignore-all
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:ignore-in-object-files:default
// LLD leaves GOT loads of unresolved symbols unrelaxed.
//#DiffIgnore:rel.extra-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).static-non-pie
//#LinkArgs:--unresolved-symbols=ignore-in-object-files
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:ignore-in-shared-libs:default
//#LinkArgs:--unresolved-symbols=ignore-in-shared-libs
//#ExpectError:foo
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:warn-unresolved-symbols:default
// LLD leaves GOT loads of unresolved symbols unrelaxed.
//#DiffIgnore:rel.extra-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).static-non-pie
//#LinkArgs:--warn-unresolved-symbols
//#ExpectWarningWild:Undefined symbol foo
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:error-unresolved-symbols:default
//#LinkArgs:--error-unresolved-symbols
//#ExpectError:foo
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

int foo();

// This weak function is just here to give us a way to avoid calling foo without the compiler
// knowing that we'll never call foo. It also helps us verify that we do the right thing with weak
// symbols, since they should be interposable.
int __attribute__((weak)) weak_fn1(void);

int __attribute__((weak, visibility(("protected")))) weak_protected(void);

int __attribute__((weak, visibility(("hidden")))) weak_hidden(void);

void _start(void) {
  runtime_init();

  if (weak_fn1) {
    foo();
  }

  if (weak_protected) {
    weak_protected();
  }

  if (weak_hidden) {
    weak_hidden();
  }

  exit_syscall(42);
}
