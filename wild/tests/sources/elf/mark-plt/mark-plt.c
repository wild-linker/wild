//#AbstractConfig:base
//#Arch:x86_64
//#RequiresGlibc:true
//#RequiresGlibcVersion:2.36
//#RequiresLinkerFlags:-z mark-plt
//#CompArgs:-fPIC
//#LinkArgs:-shared -z now
//#RunDynSym:check_plt
//#ReferenceLinkers:bfd
// BFD includes a lazy-binding PLT header and uses JUMP_SLOT even without marking.
//#DiffIgnore:.dynamic.DT_X86_64_PLTSZ
//#DiffIgnore:.dynamic.DT_X86_64_PLT
//#DiffIgnore:.dynamic.DT_RELA*

//#Config:enabled:base
//#ExpectPltMarking:true
//#LinkArgs:-z nomark-plt -z mark-plt
//#ExpectDynamic:DT_X86_64_PLT count=1
//#ExpectDynamic:DT_X86_64_PLTSZ count=1
//#ExpectDynamic:DT_X86_64_PLTENT count=1

//#Config:disabled:base
//#ExpectPltMarking:false
//#LinkArgs:-z mark-plt -z nomark-plt

//#Config:default:base
//#ExpectPltMarking:false
//#ReferenceLinkers:

//#Config:libc:enabled
//#LinkerDriver:gcc
//#CompArgs:-DUSE_LIBC

#ifdef USE_LIBC
#include <unistd.h>
#endif

int target(void) { return 42; }
int expected = 42;
static int implementation(void) { return 20; }
static int (*resolver(void))(void) { return implementation; }
static int indirect(void) __attribute__((ifunc("resolver")));

int check_plt(void) {
#ifdef USE_LIBC
  if (getpid() <= 0) return 10;
#endif
  if (indirect() != 20) return 9;
  return target() == expected ? 42 : 11;
}
