//#Config:wild-so
//#SoSingleLinker:wild
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now,-z,pack-relative-relocs
//#Shared:pack-relative-relocs-shared-1.c
//#DiffIgnore:section.rodata
//#DiffIgnore:section.data
//#ReferenceLinkers:bfd,lld
//#DiffMatchAny:true
//#CompArgs:-fPIE
//#CompSoArgs:-fPIE
//#LinkArgs:-pie

int foo(void);

int main() { return foo(); }
