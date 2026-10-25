//#AbstractConfig:base
//#CompArgs:-flto -ffat-lto-objects -O1
//#RequiresCompilerFlags:-flto -ffat-lto-objects -Werror
//#Object:runtime.c
//#ExpectSym:foo

//#AbstractConfig:clang-base:base
//#Compiler:clang
//#DiffIgnore:section.eh_frame.type

//#Config:gcc:base
//#Compiler:gcc
//#Object:foo.c
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:clang:clang-base
//#Object:foo.c
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:gcc-archive:base
//#Compiler:gcc
//#Archive:foo.c
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:clang-archive:clang-base
//#Archive:foo.c
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:gcc-thin-archive:base
//#Compiler:gcc
//#ThinArchive:foo.c
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:clang-thin-archive:clang-base
//#ThinArchive:foo.c
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:gcc-slim
//#Compiler:gcc
//#CompArgs:-flto -fno-fat-lto-objects
//#ReferenceLinkers:
//#ExpectError:(GCC slim LTO input .* was not processed by a linker plugin|Wild was compiled without linker-plugin support)
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:gcc-slim-archive
//#Compiler:gcc
//#Object:runtime.c
//#Archive:foo.c:-flto -fno-fat-lto-objects
//#ReferenceLinkers:
//#ExpectError:(GCC slim LTO input .* was not processed by a linker plugin|Wild was compiled without linker-plugin support)
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

int foo(void);

void _start(void) {
  runtime_init();
  exit_syscall(foo());
}
