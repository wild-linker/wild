//#AbstractConfig:default
// Error messages and what it takes to need a copy relocation vary by arch.
//#Arch:x86_64
//#Mode:dynamic
//#CompArgs:-fno-pic -fno-pie
//#CompSoArgs:-fPIC
//#LinkSoArgs:-no-pie -z now
//#Shared:copy-relocation-protected-lib.c
//#ReferenceLinkers:bfd,lld
//#ExpectError:(copy relocation.*protected|cannot preempt symbol)
//#ExpectError:foobar

//#Config:no-pie:default
//#LinkArgs:-no-pie

//#Config:pie:default
//#LinkArgs:-pie

extern int foobar;

void _start(void) { foobar = 42; }
