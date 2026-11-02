//#LinkerScript:regions.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x100000,paddr=0x100000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data,*],vaddr=0x200000,paddr=0x200000

int data = 42;

void _start(void) {}
