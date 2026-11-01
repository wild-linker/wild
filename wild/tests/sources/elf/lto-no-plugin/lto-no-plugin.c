// Checks what we do if we try to link LTO inputs when no plugin is supplied.

//#AbstractConfig:default
//#ReferenceLinkers:
//#CompArgs:-flto
//#RequiresCompilerFlags:-flto

//#Config:gcc:default
//#Compiler:gcc
//#CompArgs:-fno-fat-lto-objects
//#ExpectError:(GCC slim LTO input .* was not processed by a linker plugin|Wild was compiled without linker-plugin support)
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:clang:default
//#Compiler:clang
//#ExpectError:(linker plugin was not supplied|Wild was compiled without linker-plugin support)
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

void _start(void) {}
