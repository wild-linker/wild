//#Config:dup
//#ReferenceLinkers:
//#Object:duplicate_strong_symbols2.c
//#ExpectError:Duplicate symbols
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:allow-multiple-definition
//#RunEnabled:false
//#Object:duplicate_strong_symbols2.c
//#LinkArgs:--allow-multiple-definition
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:z-muldefs
//#RunEnabled:false
//#Object:duplicate_strong_symbols2.c
//#LinkArgs:-z muldefs
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:dup-lto
//#RequiresLinkerPlugin:true
//#Compiler:clang
//#LinkerDriver:clang
//#LinkArgs:-nostdlib -Wl,-znow -flto
//#ReferenceLinkers:
//#Object:duplicate_strong_symbols2.c:-flto
//#ExpectError:Duplicate symbols
//#CompArgs:-fPIE
//#LinkArgs:-pie

int test_func(void) { return 0; }

void _start(void) { test_func(); }
