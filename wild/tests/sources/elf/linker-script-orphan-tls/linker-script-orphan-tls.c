//#LinkerScript:sections.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections
//#ExpectProgramHeader:TLS sections=[.tdata,.tbss,*]
//#ExpectSym:initialized section=.tdata
//#ExpectSym:uninitialized section=.tbss

_Thread_local int initialized = 7;
_Thread_local int uninitialized;
int data = 42;

void _start(void) {}
