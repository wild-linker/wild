//#AbstractConfig:verdef
//#RunEnabled:false
//#DiffIgnore:.dynamic.DT_FLAGS*
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT
//#DiffIgnore:file-header.entry
//#Object:runtime.c

//#Config:verneed
//#Object:runtime.c
//#Object:symbol-versions-2.c
//#ExpectSym: _start section=".text"
//#ExpectSym: exit_syscall section=".text"
//#ReferenceLinkers:bfd,lld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:verdef-0:verdef
//#LinkArgs:--undefined-version
//#TestUpdateInPlace:true
//#LinkArgs:--shared --version-script=./symbol-versions-script.map
//#CompArgs:-fPIE

//#Config:verdef-1:verdef
//#LinkArgs:--undefined-version
//#LinkArgs:--shared --soname=symbol-versions.so --version-script=./symbol-versions-script.map
//#CompArgs:-fPIE

//#Config:verdef-lto:verdef
// LLD does not support GCC LTO plugins.
//#ReferenceLinkers:bfd
//#RequiresLinkerPlugin:true
//#LinkerDriver:gcc
//#CompArgs:-flto -znow
//#LinkArgs:--shared -Wl,--version-script=./symbol-versions-script.map -flto
//#DiffIgnore:eh_frame
//#CompArgs:-fPIE

//#Config:with-escaping:verdef
//#Object:escaped-glob.c
// LLD treats the backslash in this exact version-script name literally.
//#ReferenceLinkers:bfd
//#LinkArgs:--shared --version-script=./symbol-versions-with-escaping.map
//#CompArgs:-fPIE
//#ExpectSym:xfoobarx section=".text"
//#ExpectSym:pre*post section=".text"
//#ExpectSym:preapost section=".text"
//#ExpectSym:prebpost section=".text"
//#ExpectSym:prexpost section=".text"
//#ExpectSym:clsa section=".text"
//#ExpectSym:cls] section=".text"
//#ExpectSym:clsb section=".text"
//#ExpectSym:br] section=".text"
//#ExpectSym:cxxx section=".text"
//#ExpectSym:bang! section=".text"
//#ExpectSym:banga section=".text"
//#ExpectSym:bangb section=".text"
//#ExpectSym:dash- section=".text"
//#ExpectSym:dasha section=".text"
//#ExpectSym:dashm section=".text"
//#ExpectSym:dashz section=".text"
//#ExpectSym:only! section=".text"

#include "../common/runtime.h"

int foo(void);
int bar_global(void);
int bar_local(void);
int bar_v2(void);
int bar_v2_1(void);

void _start(void) {
  runtime_init();

  if (foo() != 2) {
    exit_syscall(foo());
  }
  if (bar_global() != 10) {
    exit_syscall(bar_global());
  }
  if (bar_local() != 11) {
    exit_syscall(bar_local());
  }
  if (bar_v2() != 12) {
    exit_syscall(bar_v2());
  }
  if (bar_v2_1() != 13) {
    exit_syscall(bar_v2_1());
  }

  exit_syscall(42);
}

int bar_global(void) { return 10; }

// TODO: doesn't work, the symbol is global
int bar_local(void) { return 11; }

int bar_v2(void) { return 12; }

int bar_v2_1(void) { return 13; }
