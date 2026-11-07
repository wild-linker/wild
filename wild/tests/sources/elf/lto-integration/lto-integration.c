//#Config:clang
//#RequiresLinkerPlugin:true
//#LinkerDriver:clang
//#ReferenceLinkers:lld
//#Compiler:clang
//#CompArgs:-flto
//#LinkArgs:-flto -Wl,--as-needed,-znow
//#DiffIgnore:section.rodata
//#DiffIgnore:eh_frame
//#CompArgs:-fPIE
//#LinkArgs:-pie

int main() { return 42; }
