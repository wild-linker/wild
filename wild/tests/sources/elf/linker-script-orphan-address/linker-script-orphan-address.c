//#AbstractConfig:default
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections

//#Config:text:default
//#LinkerScript:data.ld
//#LinkArgs:-Ttext=0x600000
//#ExpectSym:_start address=0x600000

//#Config:data:default
//#LinkerScript:text.ld
//#LinkArgs:-Tdata=0x600000
//#ExpectSym:data address=0x600000

//#Config:bss:default
//#LinkerScript:text.ld
//#LinkArgs:-Tbss=0x600000
//#ExpectSym:bss address=0x600000

int data = 42;
int bss;

void _start(void) {}
