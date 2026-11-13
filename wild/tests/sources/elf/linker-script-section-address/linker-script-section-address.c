//#LinkerScript:section-address.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections
//#ExpectSym:_start section=.text
//#ExpectSym:data address=0x800000

int data = 42;

void _start(void) {}
