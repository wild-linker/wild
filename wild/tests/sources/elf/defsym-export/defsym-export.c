//#AbstractConfig:exe
//#Mode:dynamic
//#Object:runtime.c
//#Shared:empty.c
//#CompArgs:-fPIE
//#CompSoArgs:-fPIC
//#LinkArgs:-z now -no-pie
//#PostLinkArgs:--defsym=prov=w
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#DiffIgnore:.dynamic.DT_NEEDED

//#Config:unexported:exe
//#NoDynSym:prov

//#Config:export-dynamic:exe
//#LinkArgs:--export-dynamic
//#ExpectDynSym:prov

//#Config:export-symbol:exe
//#LinkArgs:--export-dynamic-symbol=prov
//#ExpectDynSym:prov
//#NoDynSym:w

//#Config:needed-by-dso:exe
//#Shared:needed.c
//#LinkArgs:--no-as-needed
//#ExpectDynSym:prov
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT

//#Config:shared
//#CompArgs:-fPIC -DNO_RUNTIME
//#LinkArgs:-shared -z now
//#PostLinkArgs:--defsym=prov=w
//#RunEnabled:false
//#ExpectDynSym:prov
//#ReferenceLinkers:bfd,lld
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT

#ifdef NO_RUNTIME
void runtime_init(void) {}
void exit_syscall(int code) { (void)code; }
#else
#include "../common/runtime.h"
#endif

int w(void) { return 2; }

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
