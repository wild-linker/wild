//#LinkerScript:sections.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections
//#ExpectProgramHeader:TLS mem-size=8192
//#ExpectSym:data section=.data

_Thread_local char tls[8192];
int data = 42;

void _start(void) {}
