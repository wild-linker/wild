//#AbstractConfig:base
//#Arch:x86_64
//#RequiresGlibc:true
//#RequiresGlibcVersion:2.36
//#RequiresLinkerFlags:-z mark-plt
//#LinkerDriver:gcc
//#CompArgs:-fPIC -DUSE_LIBC
//#LinkArgs:-Wl,-z,mark-plt,-z,now,--gc-sections
//#ReferenceLinkers:bfd
//#DiffIgnore:.dynamic.DT_X86_64_PLT
//#DiffIgnore:.dynamic.DT_X86_64_PLTSZ
//#DiffIgnore:.dynamic.DT_RELA*
//#ExpectPltMarking:true
//#ExpectDynamic:DT_X86_64_PLT
//#ExpectDynamic:DT_X86_64_PLTSZ
//#ExpectDynamic:DT_X86_64_PLTENT

//#Config:pie:base
//#LinkArgs:-pie

//#Config:non-pie:base
//#LinkArgs:-no-pie

#include "../mark-plt/mark-plt.c"

int main(void) { return check_plt(); }
