// Wild and BFD use an absolute move where LLD uses a PC-relative lea.
//#DiffIgnore:rel.different-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).MovIndirectToLea.static-non-pie
// Tests that library search order is directory-first, with shared objects preferred only within
// each directory.

//#CompArgs:-fPIC
//#CompSoArgs:-fPIC
//#Object:runtime.c
//#Mode:unspecified
//#SoSingleLinker:ld
//#Archive:as(first/libfoo.a),noadd:from-archive.c
//#Shared:as(second/libfoo.so),template(-L$OUT_DIR/first -L$OUT_DIR/second -lfoo):from-shared.c
//#ExpectSym:selected_from_archive
//#LinkArgs:-no-pie

#include "../common/runtime.h"

int foo(void);

void _start(void) {
  runtime_init();
  exit_syscall(foo());
}
