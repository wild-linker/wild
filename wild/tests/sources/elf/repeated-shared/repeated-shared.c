//#AbstractConfig:base
//#Object:runtime.c
//#Mode:dynamic
//#CompArgs:-fPIC
//#SoSingleLinker:ld
//#Shared:as(libbaseline.so):force-dynamic-linking.c
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT
//#ReferenceLinkers:bfd,lld

//#Config:as-needed-then-required:base
//#Shared:as(librepeated.so),template(--push-state --as-needed $O --no-as-needed $O --pop-state):library.c
//#ExpectDynamic:DT_NEEDED count=2
//#CompSoArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:required-then-as-needed:base
//#Shared:as(librepeated.so),template(--push-state --no-as-needed $O --as-needed $O --pop-state):library.c
//#ExpectDynamic:DT_NEEDED count=2
//#CompSoArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:required-twice:base
//#Shared:as(librepeated.so),template(--push-state --no-as-needed $O $O --pop-state):library.c
//#ExpectDynamic:DT_NEEDED count=2
//#CompSoArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:unused-twice:base
//#Shared:as(librepeated.so),template(--push-state --as-needed $O $O --pop-state):library.c
//#ExpectDynamic:DT_NEEDED count=1
//#CompSoArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:used-twice:base
//#CompArgs:-DREFERENCE_LIBRARY
//#Shared:as(librepeated.so),template(--push-state --as-needed $O $O --pop-state):library.c
//#ExpectDynamic:DT_NEEDED count=2
//#CompSoArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:required-by-script:base
//#DiffIgnore:.dynamic.DT_NEEDED
//#LinkArgs:-L$OUT_DIR -rpath $OUT_DIR
//#Shared:as(librepeated.so),template(--push-state --as-needed $O --pop-state):library.c
//#AugmentLinkerScript:template(--push-state --no-as-needed $O --pop-state):library.ld
//#ExpectDynamic:DT_NEEDED count=2
//#CompSoArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:required-within-script:base
//#DiffIgnore:.dynamic.DT_NEEDED
//#LinkArgs:-L$OUT_DIR -rpath $OUT_DIR
//#Shared:as(librepeated.so),noadd:library.c
//#AugmentLinkerScript:repeated-library.ld
//#ExpectDynamic:DT_NEEDED count=2
//#CompSoArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

extern int repeated_value;

void _start(void) {
  runtime_init();
#ifdef REFERENCE_LIBRARY
  exit_syscall(repeated_value);
#else
  exit_syscall(42);
#endif
}
