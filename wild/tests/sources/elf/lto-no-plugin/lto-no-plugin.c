// Checks what we do if we try to link LTO inputs when no plugin is supplied.

//#AbstractConfig:default
//#ReferenceLinkers:
//#ExpectError:linker plugin was not supplied
//#CompArgs:-flto
//#RequiresLinkerPlugin:true

//#Config:gcc:default
//#Compiler:gcc
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:clang:default
//#Compiler:clang
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

void _start(void) {}
