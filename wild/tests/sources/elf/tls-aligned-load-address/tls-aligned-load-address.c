//#LinkerScript:sections.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections
//#ExpectProgramHeader:TLS sections=[.tdata,.tbss,*]

_Thread_local int initialized = 7;
_Thread_local char uninitialized[256] __attribute__((aligned(256)));
int data = 42;

void _start(void) {}
