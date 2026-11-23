//#LinkerScript:regions.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-unwind-tables -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie -z max-page-size=0x10000
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x100000,paddr=0x100000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.first,.bss,*],vaddr=0x200000,paddr=0x200000
//#MaxFileSize:150000

int data __attribute__((section(".first"))) = 42;
int bss;

void _start(void) {}
