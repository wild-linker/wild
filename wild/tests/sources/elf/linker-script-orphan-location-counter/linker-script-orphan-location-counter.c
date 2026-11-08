//#AbstractConfig:default
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections

//#Config:tls:default
//#CompArgs:-DTEST_TLS
//#LinkerScript:tls.ld
//#ExpectSym:initialized section=.tdata
//#ExpectSym:uninitialized section=.tbss

//#Config:data:default
//#LinkerScript:data.ld
//#ExpectSym:initialized section=.orphan
//#ExpectSym:uninitialized section=.bss

#ifdef TEST_TLS
_Thread_local int initialized = 7;
_Thread_local int uninitialized;
int data = 42;
#else
__attribute__((section(".orphan"))) int initialized = 7;
int uninitialized;
#endif

void _start(void) {}
