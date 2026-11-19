//#AbstractConfig:base
//#RequiresLinkerPlugin:true
//#CompArgs:-O2 -fno-lto -fPIC
//#LinkArgs:-flto -nostdlib -shared -Wl,-z,now
//#RunEnabled:false
//#ExpectSym:excluded
//#NoDynSym:excluded
//#ExpectSym:excluded_data
//#NoDynSym:excluded_data
//#ExpectDynSym:included
//#ExpectDynSym:dependency
//#ExpectDynSym:call
//#ExpectDynSym:address
//#NoSym:unused

//#AbstractConfig:clang-base:base
//#Compiler:clang
//#LinkerDriver:clang
//#ReferenceLinkers:lld

//#Config:clang:clang-base
//#Archive:excluded.c:-flto,unused.c:-flto
//#Object:included.c:-flto
//#LinkArgs:-Wl,--exclude-libs=excluded.a

//#Config:clang-thin:clang-base
//#Archive:excluded.c:-flto=thin,unused.c:-flto=thin
//#Object:included.c:-flto=thin
//#LinkArgs:-Wl,--exclude-libs=excluded.a

//#Config:gcc:base
// BFD hides the strong definition when the excluded archive has a weak alternative.
//#CompArgs:-DNO_WEAK_OVERRIDE
//#Compiler:gcc
//#LinkerDriver:gcc
//#ReferenceLinkers:bfd
//#Archive:excluded.c:-flto,unused.c:-flto
//#Object:included.c:-flto
//#LinkArgs:-Wl,--exclude-libs=excluded.a

//#Config:clang-all:clang
//#LinkArgs:-Wl,--exclude-libs=ALL

//#Config:clang-thin-all:clang-thin
//#LinkArgs:-Wl,--exclude-libs=ALL

//#Config:gcc-all:gcc
//#LinkArgs:-Wl,--exclude-libs=ALL

int excluded(void);
int included(void);
extern int excluded_data;

int* address = &excluded_data;

int dependency(void) { return 2; }

int call(void) { return excluded() + included(); }
