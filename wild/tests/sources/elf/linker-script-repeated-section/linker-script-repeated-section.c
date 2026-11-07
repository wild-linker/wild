//#LinkerScript:repeated.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections
// Wild emits two .text sections and two RX segments, matching LLD. BFD uses RWX.
//#DiffIgnore:segment.LOAD.RX.alignment
//#ExpectSym:_start section=.text
//#ExpectSym:second section=.text
//#ExpectSym:data section=.data

int data = 42;

void _start(void) {}

__attribute__((section(".text.second"))) int second(void) { return 23; }
