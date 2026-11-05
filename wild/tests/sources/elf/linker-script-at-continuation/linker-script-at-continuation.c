//#LinkerScript:sections.ld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections

__attribute__((section(".foo"))) int foo = 1;
__attribute__((section(".bar"))) int bar = 2;

void _start(void) {}
