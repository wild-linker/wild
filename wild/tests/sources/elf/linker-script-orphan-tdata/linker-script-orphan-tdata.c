//#LinkerScript:sections.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections
//#ExpectProgramHeader:TLS sections=[.tdata,.tdata.orphan,.tbss,*]
//#ExpectSym:regular_tdata section=.tdata
//#ExpectSym:orphan_tdata section=.tdata.orphan
//#ExpectSym:uninit_tbss section=.tbss
//#ExpectSym:data section=.data

_Thread_local int regular_tdata = 1;
__attribute__((section(".tdata.orphan"))) _Thread_local int orphan_tdata = 2;
_Thread_local int uninit_tbss;
int data = 42;

void _start(void) {}
