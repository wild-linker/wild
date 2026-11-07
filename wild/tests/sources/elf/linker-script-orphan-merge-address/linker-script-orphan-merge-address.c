//#LinkerScript:sections.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections --section-start=.foo=0x600000
//#ExpectSym:orphan section=.foo
//#ExpectSym:matched section=.foo

__attribute__((section(".foo"))) int orphan = 42;
__attribute__((section(".foo.other"))) int matched = 24;

void _start(void) {}
