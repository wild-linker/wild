//#AbstractConfig:base
//#RequiresLinkerPlugin:true
//#Compiler:clang
//#LinkerDriver:clang
//#ExpectError:bogus_insn
//#LinkArgs:-nostdlib -no-pie

//#Config:thin:base
//#CompArgs:-O1 -flto=thin -fno-pie
//#LinkArgs:-flto=thin -Wl,-plugin-opt=jobs=1

//#Config:full:base
//#CompArgs:-O1 -flto -fno-pie
//#LinkArgs:-flto

void bad(void) { __asm__ volatile("bogus_insn"); }

void _start(void) { bad(); }
